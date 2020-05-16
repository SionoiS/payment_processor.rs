use std::collections::HashMap;
use std::sync::Mutex;

use actix_web::post;
use actix_web::{web, HttpResponse, Responder};

use crate::models::Error;
use crate::models::ErrorMessage;
use crate::models::Message;
use crate::MyData;

use firestore_grpc_cloudrun::{
    value::ValueType, CreateDocumentRequest, Document, DocumentMask, GetDocumentRequest,
    UpdateDocumentRequest, Value,
};

use tonic::Code;

const USER_ERROR: ErrorMessage = ErrorMessage {
    error: Error {
        code: "INVALID_USER",
        message: "Invalid user",
    },
};

const INCORRECT_INVOICE: ErrorMessage = ErrorMessage {
    error: Error {
        code: "INCORRECT_INVOICE",
        message: "Incorrect invoice",
    },
};

#[post("/webhook")]
async fn notifications(
    firestore: web::Data<Mutex<MyData>>,
    notif: web::Json<Message>,
) -> impl Responder {
    let firestore = firestore.lock();
    let mut firestore = match firestore {
        Ok(firestore) => firestore,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    match notif.into_inner() {
        Message::UserValidation { user } => {
            let req = GetDocumentRequest {
                name: format!(
                    "projects/{}/databases/(default)/documents/users/{}",
                    firestore.project_id, user.id
                ),
                mask: None,
                consistency_selector: None,
            };

            if let Err(error) = firestore.client.get_document(req).await {
                if let Code::NotFound = error.code() {
                    return HttpResponse::BadRequest().json(USER_ERROR);
                } else {
                    return HttpResponse::InternalServerError().finish();
                }
            }

            HttpResponse::Ok().finish()
        }
        Message::Payment {
            purchase,
            user,
            transaction,
        } => {
            let req = GetDocumentRequest {
                name: format!(
                    "projects/{}/databases/(default)/documents/users/{}",
                    firestore.project_id, user.id
                ),
                mask: Some(DocumentMask {
                    field_paths: vec!["Credits".to_owned()],
                }),
                consistency_selector: None,
            };

            let user_doc = firestore.client.get_document(req).await;
            let mut user_doc = match user_doc {
                Ok(user_doc) => user_doc.into_inner(),
                Err(error) => {
                    if let Code::NotFound = error.code() {
                        return HttpResponse::BadRequest().json(USER_ERROR);
                    } else {
                        return HttpResponse::InternalServerError().finish();
                    }
                }
            };

            let req = GetDocumentRequest {
                name: format!(
                    "projects/{}/databases/(default)/documents/users/{}/transact/{}",
                    firestore.project_id, user.id, transaction.id
                ),
                mask: None,
                consistency_selector: None,
            };

            //transaction already processed do nothing
            if firestore.client.get_document(req).await.is_ok() {
                return HttpResponse::Ok().finish();
            }

            let mut data: HashMap<String, Value> = HashMap::with_capacity(3);

            data.insert(
                "Currency".to_owned(),
                Value {
                    value_type: Some(ValueType::StringValue(purchase.virtual_currency.currency)),
                },
            );

            data.insert(
                "Cost".to_owned(),
                Value {
                    value_type: Some(ValueType::IntegerValue(purchase.virtual_currency.amount)),
                },
            );

            data.insert(
                "Quantity".to_owned(),
                Value {
                    value_type: Some(ValueType::IntegerValue(purchase.virtual_currency.quantity)),
                },
            );

            let doc = Document {
                name: format!(
                    "projects/{}/databases/(default)/documents/users/{}/transact/{}",
                    firestore.project_id, user.id, transaction.id
                ),
                fields: data,
                create_time: None,
                update_time: None,
            };

            let req = CreateDocumentRequest {
                parent: format!(
                    "projects/{}/databases/(default)/documents/users/{}",
                    firestore.project_id, user.id
                ),
                collection_id: "transact".to_owned(),
                document_id: transaction.id.to_string(),
                document: Some(doc),
                mask: None,
            };

            if firestore.client.create_document(req).await.is_err() {
                return HttpResponse::InternalServerError().finish();
            }

            //Increment credit in user document
            if let Some(credits) = user_doc.fields.get_mut("Credits") {
                if let Some(credits) = credits.value_type.as_mut() {
                    if let ValueType::IntegerValue(credits) = credits {
                        *credits += purchase.virtual_currency.quantity
                    }
                }
            }

            let req = UpdateDocumentRequest {
                document: Some(user_doc),
                update_mask: Some(DocumentMask {
                    field_paths: vec!["Credits".to_owned()],
                }),
                mask: Some(DocumentMask {
                    field_paths: vec!["Credits".to_owned()],
                }),
                current_document: None,
            };

            if firestore.client.update_document(req).await.is_err() {
                return HttpResponse::InternalServerError().finish();
            }

            HttpResponse::Ok().finish()
        }
        Message::Refund {
            purchase,
            user,
            transaction,
            refund_details,
        } => {
            let req = GetDocumentRequest {
                name: format!(
                    "projects/{}/databases/(default)/documents/users/{}",
                    firestore.project_id, user.id
                ),
                mask: Some(DocumentMask {
                    field_paths: vec!["Credits".to_owned()],
                }),
                consistency_selector: None,
            };

            let user_doc = firestore.client.get_document(req).await;
            let mut user_doc = match user_doc {
                Ok(user_doc) => user_doc.into_inner(),
                Err(error) => {
                    if let Code::NotFound = error.code() {
                        return HttpResponse::BadRequest().json(USER_ERROR);
                    } else {
                        return HttpResponse::InternalServerError().finish();
                    }
                }
            };

            let req = GetDocumentRequest {
                name: format!(
                    "projects/{}/databases/(default)/documents/users/{}/transact/{}",
                    firestore.project_id, user.id, transaction.id
                ),
                mask: None,
                consistency_selector: None,
            };

            let transact_doc = firestore.client.get_document(req).await;
            let mut transact_doc = match transact_doc {
                Ok(transact_doc) => transact_doc.into_inner(),
                Err(error) => {
                    if let Code::NotFound = error.code() {
                        return HttpResponse::BadRequest().json(INCORRECT_INVOICE);
                    } else {
                        return HttpResponse::InternalServerError().finish();
                    }
                }
            };

            transact_doc.fields.insert(
                "RefundDate".to_owned(),
                Value {
                    value_type: Some(ValueType::TimestampValue(prost_types::Timestamp::from(
                        std::time::SystemTime::now(),
                    ))),
                },
            );

            transact_doc.fields.insert(
                "RefundCode".to_owned(),
                Value {
                    value_type: Some(ValueType::IntegerValue(refund_details.code)),
                },
            );

            let req = UpdateDocumentRequest {
                document: Some(transact_doc),
                update_mask: Some(DocumentMask {
                    field_paths: vec!["RefundDate".to_owned(), "RefundCode".to_owned()],
                }),
                mask: Some(DocumentMask {
                    field_paths: vec!["RefundDate".to_owned(), "RefundCode".to_owned()],
                }),
                current_document: None,
            };

            if firestore.client.update_document(req).await.is_err() {
                return HttpResponse::InternalServerError().finish();
            }

            //Decrement credit in user document
            if let Some(credits) = user_doc.fields.get_mut("Credits") {
                if let Some(credits) = credits.value_type.as_mut() {
                    if let ValueType::IntegerValue(credits) = credits {
                        *credits -= purchase.virtual_currency.quantity
                    }
                }
            }

            let req = UpdateDocumentRequest {
                document: Some(user_doc),
                update_mask: Some(DocumentMask {
                    field_paths: vec!["Credits".to_owned()],
                }),
                mask: Some(DocumentMask {
                    field_paths: vec!["Credits".to_owned()],
                }),
                current_document: None,
            };

            if firestore.client.update_document(req).await.is_err() {
                return HttpResponse::InternalServerError().finish();
            }

            HttpResponse::Ok().finish()
        }
    }
}
