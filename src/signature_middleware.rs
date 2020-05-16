use actix_service::{Service, Transform};
use actix_web::{
    dev::ServiceRequest, dev::ServiceResponse, http::header, http::header::HeaderValue, Error,
    HttpMessage, HttpResponse,
};
use futures::future::{ok, Future, Ready};
use futures::stream::StreamExt;
use sha1::{Digest, Sha1};
use std::cell::RefCell;
use std::env;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

fn get_secret_key() -> String {
    let secret = env::var("WEBHOOK_SECRET_KEY")
        .expect("Trying to read enviroment variable WEBHOOK_SECRET_KEY Error: ");

    if secret.len() != 20 {
        panic!("WEBHOOK_SECRET_KEY must be 20 characters long");
    }

    secret
}

pub struct VerifySignature;

impl<S: 'static, B> Transform<S> for VerifySignature
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = VerifySignatureMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(VerifySignatureMiddleware {
            service: Rc::new(RefCell::new(service)),
            secret_key: get_secret_key(),
        })
    }
}

fn extract_signature(header_value: &HeaderValue) -> Option<[u8; 20]> {
    if let Ok(sig) = header_value.to_str() {
        //sig == "Bearer 40 char"
        if let Some(sig) = sig.get(7..47) {
            //sig == "40 char"
            if let Ok(decoded) = hex::decode(sig) {
                //https://docs.rs/crate/hex
                if decoded.len() == 20 {
                    let mut result = [0; 20];

                    result.copy_from_slice(&decoded);
                    return Some(result);
                }
            }
        }
    }

    None
}

use crate::models::Error as JsonError;
use crate::models::ErrorMessage;

const SIGNATURE_ERROR: ErrorMessage = ErrorMessage {
    error: JsonError {
        code: "INVALID_SIGNATURE",
        message: "Invalid Signature",
    },
};

pub struct VerifySignatureMiddleware<S> {
    service: Rc<RefCell<S>>, //Rc & RefCell why???
    secret_key: String,
}

impl<S, B> Service for VerifySignatureMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        let header_value = req.headers().get(header::AUTHORIZATION);
        let header_value = match header_value {
            Some(bearer) => bearer,
            None => {
                return Box::pin(ok(req.into_response(
                    HttpResponse::Unauthorized()
                        .json(SIGNATURE_ERROR)
                        .into_body(),
                )));
            }
        };

        let signature = extract_signature(header_value);
        let signature = match signature {
            Some(sig) => sig,
            None => {
                return Box::pin(ok(req.into_response(
                    HttpResponse::Unauthorized()
                        .json(SIGNATURE_ERROR)
                        .into_body(),
                )));
            }
        };

        let secret = self.secret_key.clone();
        let mut svc = self.service.clone();

        Box::pin(async move {
            let mut hasher = Sha1::new();

            let mut stream = req.take_payload();

            while let Some(chunk) = stream.next().await {
                hasher.input(chunk?);
            }

            hasher.input(secret.as_bytes());

            let hash = hasher.result();

            if signature != hash.as_slice() {
                return Ok(req.into_response(
                    HttpResponse::Unauthorized()
                        .json(SIGNATURE_ERROR)
                        .into_body(),
                ));
            }

            Ok(svc.call(req).await?)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers;
    use actix_web::http::header;
    use actix_web::http::StatusCode;
    use actix_web::test;
    use actix_web::test::TestRequest;
    use actix_web::App;

    #[actix_rt::test]
    async fn wrong_signature() {
        let app = App::new()
            .wrap(VerifySignature)
            .service(handlers::notifications);
        let mut app = test::init_service(app).await;

        let data = "examplepayload";

        let req = TestRequest::post()
            .uri("/webhook")
            .header(
                header::AUTHORIZATION,
                "Bearer bd31a2212735b01bc15e8350a6d27003a2b63d26", //hash of "examplepayloadUltra1Top2Secret3Key" with last hex changed
            )
            .set_payload(data)
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_rt::test]
    async fn correct_signature() {
        let app = App::new()
            .wrap(VerifySignature)
            .service(handlers::notifications);
        let mut app = test::init_service(app).await;

        let data = "examplepayload";

        let req = TestRequest::post()
            .uri("/webhook")
            .header(
                header::AUTHORIZATION,
                "Bearer bd31a2212735b01bc15e8350a6d27003a2b63d27", //hash of "examplepayloadUltra1Top2Secret3Key"
            )
            .set_payload(data)
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
