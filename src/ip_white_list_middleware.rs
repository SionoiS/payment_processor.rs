use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, HttpResponse};
use futures::future::{ok, Ready};
use futures::Future;
use ipnet::IpNet;
use std::env;
use std::pin::Pin;
use std::task::{Context, Poll};

fn get_white_list() -> Vec<IpNet> {
    //185.30.20.0/24;185.30.21.0/24;...etc
    let ips = env::var("IP_WHITE_LIST")
        .expect("Trying to read enviroment variable IP_WHITE_LIST Error: ");

    let ips: Vec<IpNet> = ips
        .split(';')
        .map(|ip| ip.parse().expect("Trying to parse ip Error: "))
        .collect();

    ips
}

pub struct IpWhiteList;

impl<S, B> Transform<S> for IpWhiteList
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = IpWhiteListMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(IpWhiteListMiddleware {
            service,
            white_list: get_white_list(),
        })
    }
}

pub struct IpWhiteListMiddleware<S> {
    service: S,
    white_list: Vec<IpNet>,
}

impl<S, B> Service for IpWhiteListMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
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

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        if let Some(socket) = req.peer_addr() {
            let remote_ip = socket.ip();

            for ip in &self.white_list {
                if ip.contains(&remote_ip) {
                    let fut = self.service.call(req);

                    return Box::pin(async move {
                        let res = fut.await?;
                        Ok(res)
                    });
                }
            }
        }

        Box::pin(ok(req.into_response(
            HttpResponse::Unauthorized().finish().into_body(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers;
    use actix_web::http::StatusCode;
    use actix_web::test;
    use actix_web::test::TestRequest;
    use actix_web::App;
    use std::net::SocketAddr;

    #[actix_rt::test]
    async fn wrong_ip() {
        let app = App::new()
            .wrap(IpWhiteList)
            .service(handlers::notifications);
        let mut app = test::init_service(app).await;

        let socket = "0.0.0.0:8080".parse::<SocketAddr>().unwrap();

        let req = TestRequest::post()
            .uri("/webhook")
            .peer_addr(socket)
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_rt::test]
    async fn correct_ip() {
        let app = App::new()
            .wrap(IpWhiteList)
            .service(handlers::notifications);
        let mut app = test::init_service(app).await;

        let socket = "185.30.21.255:8080".parse::<SocketAddr>().unwrap();

        let req = TestRequest::post()
            .uri("/webhook")
            .peer_addr(socket)
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
