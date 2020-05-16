use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Deserialize)]
#[serde(tag = "notification_type")]
pub enum Message {
    #[serde(rename = "user_validation")]
    UserValidation { user: User },
    #[serde(rename = "payment")]
    Payment {
        purchase: Purchase,
        user: User,
        transaction: Transaction,
        //payment_details: PaymentDetails,
    },
    #[serde(rename = "refund")]
    Refund {
        purchase: Purchase,
        user: User,
        transaction: Transaction,
        refund_details: RefundDetails,
        //payment_details: PaymentDetails,
    },
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct PaymentDetails {
    #[serde(rename = "xsolla_fee")]
    xsolla_fee: Option<Payment>,

    #[serde(rename = "payout")]
    payout: Option<Payment>,

    #[serde(rename = "vat")]
    vat: Option<Payment>,

    #[serde(rename = "payout_currency_rate")]
    payout_currency_rate: Option<i64>,

    #[serde(rename = "payment_method_fee")]
    payment_method_fee: Option<Payment>,

    #[serde(rename = "payment")]
    payment: Option<Payment>,

    #[serde(rename = "repatriation_commission")]
    repatriation_commission: Option<Payment>,
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct Payment {
    #[serde(rename = "currency")]
    currency: Option<String>,

    #[serde(rename = "amount")]
    amount: Option<i64>,
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct Purchase {
    #[serde(rename = "virtual_currency")]
    pub virtual_currency: VirtualCurrency,
    //#[serde(rename = "subscription")]
    //subscription: Option<Subscription>,

    //#[serde(rename = "checkout")]
    //checkout: Option<Payment>,

    //#[serde(rename = "virtual_items")]
    //virtual_items: Option<VirtualItems>,

    //#[serde(rename = "total")]
    //total: Option<Payment>,

    //#[serde(rename = "promotions")]
    //promotions: Option<Vec<Promotion>>,

    //#[serde(rename = "coupon")]
    //coupon: Option<Coupon>,
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct Coupon {
    #[serde(rename = "coupon_code")]
    coupon_code: Option<String>,

    #[serde(rename = "campaign_code")]
    campaign_code: Option<String>,
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct Promotion {
    #[serde(rename = "technical_name")]
    technical_name: Option<String>,

    #[serde(rename = "id")]
    id: Option<i64>,
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct Subscription {
    #[serde(rename = "plan_id")]
    plan_id: Option<String>,

    #[serde(rename = "subscription_id")]
    subscription_id: Option<i64>,

    #[serde(rename = "date_create")]
    date_create: Option<String>,

    #[serde(rename = "date_next_charge")]
    date_next_charge: Option<String>,

    #[serde(rename = "currency")]
    currency: Option<String>,

    #[serde(rename = "amount")]
    amount: Option<f64>,
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct VirtualCurrency {
    //#[serde(rename = "name")]
    //name: Option<String>,

    //#[serde(rename = "sku")]
    //sku: Option<String>,
    #[serde(rename = "quantity")]
    pub quantity: i64,

    #[serde(rename = "currency")]
    pub currency: String,

    #[serde(rename = "amount")]
    pub amount: i64,
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct VirtualItems {
    #[serde(rename = "items")]
    items: Option<Vec<Item>>,

    #[serde(rename = "currency")]
    currency: Option<String>,

    #[serde(rename = "amount")]
    amount: Option<i64>,
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct Item {
    #[serde(rename = "sku")]
    sku: Option<String>,

    #[serde(rename = "amount")]
    amount: Option<i64>,
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct RefundDetails {
    #[serde(rename = "code")]
    pub code: i64,
    //#[serde(rename = "reason")]
    //reason: Option<String>,
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "id")]
    pub id: i64,
    //#[serde(rename = "external_id")]
    //external_id: Option<String>,

    //#[serde(rename = "dry_run")]
    //dry_run: Option<i64>,

    //#[serde(rename = "agreement")]
    //agreement: Option<i64>,
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct User {
    //#[serde(rename = "ip")]
    //ip: Option<String>,

    //#[serde(rename = "phone")]
    //phone: Option<String>,

    //#[serde(rename = "email")]
    //email: Option<String>,
    #[serde(rename = "id")]
    pub id: String,
    //#[serde(rename = "name")]
    //name: Option<String>,

    //#[serde(rename = "country")]
    //country: Option<String>,
}

#[derive(PartialEq, Debug, Serialize)]
pub struct ErrorMessage<'a> {
    #[serde(rename = "error")]
    pub error: Error<'a>,
}

#[derive(PartialEq, Debug, Serialize)]
pub struct Error<'a> {
    #[serde(rename = "code")]
    pub code: &'a str,

    #[serde(rename = "message")]
    pub message: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_validation_deserialize() {
        let json = r#"
        {
            "notification_type": "user_validation",
            "user": {
                "ip": "127.0.0.1",
                "phone": "18777976552",
                "email": "email@example.com",
                "id": "1234567",
                "name": "Xsolla User",
                "country": "US"
            }
        }"#;

        let user = User {
            id: String::from("1234567"),
        };

        let data = Message::UserValidation { user };

        let msg = serde_json::from_str::<Message>(json).unwrap();

        assert_eq!(data, msg)
    }

    #[test]
    fn payment_deserialize() {
        let json = r#"
        {
            "notification_type": "payment",
            "purchase": {
                "virtual_currency": {
                    "name": "Coins",
                    "sku": "test_package1",
                    "quantity": 10,
                    "currency": "USD",
                    "amount": 100
                },
                "subscription": {
                    "plan_id": "b5dac9c8",
                    "subscription_id": 10,
                    "product_id": "Demo Product",
                    "date_create": "2014-09-22T19:25:25+04:00",
                    "date_next_charge": "2014-10-22T19:25:25+04:00",
                    "currency": "USD",
                    "amount": 9.99
                },
                "checkout": {
                    "currency": "USD",
                    "amount": 50
                },
                "virtual_items": {
                    "items": [
                        {
                            "sku": "test_item1",
                            "amount": 1
                        }
                    ],
                    "currency": "USD",
                    "amount": 50
                },
                "total": {
                    "currency": "USD",
                    "amount": 200
                },
                "promotions": [
                    {
                        "technical_name": "Demo Promotion",
                        "id": 853
                    }
                ],
                "coupon": {
                    "coupon_code": "ICvj45S4FUOyy",
                    "campaign_code": "1507"
                }
            },
            "user": {
                "ip": "127.0.0.1",
                "phone": "18777976552",
                "email": "email@example.com",
                "id": "1234567",
                "name": "Xsolla User",
                "country": "US"
            },
            "transaction": {
                "id": 1,
                "external_id": "1",
                "payment_date": "2014-09-24T20:38:16+04:00",
                "payment_method": 1,
                "dry_run": 1,
                "agreement": 1
            },
            "payment_details": {
                "payment": {
                    "currency": "USD",
                    "amount": 230
                },
                "vat": {
                    "currency": "USD",
                    "amount": 0
                },
                "payout_currency_rate": 1,
                "payout": {
                    "currency": "USD",
                    "amount": 200
                },
                "xsolla_fee": {
                    "currency": "USD",
                    "amount": 10
                },
                "payment_method_fee": {
                    "currency": "USD",
                    "amount": 20
                },
                "repatriation_commission": {
                    "currency": "USD",
                    "amount": 10
                }
            }
        }
        "#;

        let purchase = Purchase {
            virtual_currency: VirtualCurrency {
                currency: String::from("USD"),
                quantity: 10,
                amount: 100,
            },
        };

        let user = User {
            id: String::from("1234567"),
        };

        let transaction = Transaction { id: 1 };

        let data = Message::Payment {
            purchase,
            user,
            transaction,
        };

        let msg = serde_json::from_str::<Message>(json).unwrap();

        assert_eq!(data, msg)
    }

    #[test]
    fn refund_deserialize() {
        let json = r#"
        {
            "notification_type": "refund",
            "purchase": {
                "virtual_currency": {
                    "name": "Coins",
                    "quantity": 10,
                    "currency": "USD",
                    "amount": 100
                },
                "subscription": {
                    "plan_id": "b5dac9c8",
                    "subscription_id": 10,
                    "date_create": "2014-09-22T19:25:25+04:00",
                    "currency": "USD",
                    "amount": 9.99
                },
                "checkout": {
                    "currency": "USD",
                    "amount": 50
                },
                "virtual_items": {
                    "items": [
                        {
                            "sku": "test_item1",
                            "amount": 1
                        }
                    ],
                    "currency": "USD",
                    "amount": 50
                },
                "total": {
                    "currency": "USD",
                    "amount": 200
                }
            },
            "user": {
                "ip": "127.0.0.1",
                "phone": "18777976552",
                "email": "email@example.com",
                "id": "1234567",
                "name": "Xsolla User",
                "country": "US"
            },
            "transaction": {
                "id": 1,
                "external_id": "1",
                "dry_run": 1,
                "agreement": 1
            },
            "refund_details": {
                "code": 1,
                "reason": "Fraud"
            },
            "payment_details": {
                "xsolla_fee": {
                    "currency": "USD",
                    "amount": 10
                },
                "payout": {
                    "currency": "USD",
                    "amount": 200
                },
                "payment_method_fee": {
                    "currency": "USD",
                    "amount": 20
                },
                "payment": {
                    "currency": "USD",
                    "amount": 230
                },
                "repatriation_commission": {
                    "currency": "USD",
                    "amount": 10
                }
            }
        }
        "#;

        let purchase = Purchase {
            virtual_currency: VirtualCurrency {
                currency: String::from("USD"),
                quantity: 10,
                amount: 100,
            },
        };

        let user = User {
            id: String::from("1234567"),
        };

        let transaction = Transaction { id: 1 };

        let refund_details = RefundDetails { code: 1 };

        let data = Message::Refund {
            purchase,
            user,
            transaction,
            refund_details,
        };

        let msg = serde_json::from_str::<Message>(json).unwrap();

        assert_eq!(data, msg)
    }

    #[test]
    fn error_serialize() {
        let json = r#"{"error":{"code":"INVALID_USER","message":"Invalid user"}}"#;

        let error = Error {
            code: "INVALID_USER",
            message: "Invalid user",
        };

        let data = ErrorMessage { error };

        let msg = serde_json::to_string::<ErrorMessage>(&data).unwrap();

        assert_eq!(json, msg)
    }
}
