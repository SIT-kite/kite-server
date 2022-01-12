use poem::{Endpoint, IntoResponse, Middleware, Request, Response, Result};

pub struct Logger;

impl<E: Endpoint> Middleware<E> for Logger {
    type Output = LogImpl<E>;

    fn transform(&self, ep: E) -> Self::Output {
        LogImpl(ep)
    }
}

pub struct LogImpl<E>(E);

#[poem::async_trait]
impl<E: Endpoint> Endpoint for LogImpl<E> {
    type Output = Response;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        println!("request: {}", req.uri().path());
        let res = self.0.call(req).await;

        match res {
            Ok(resp) => {
                let resp = resp.into_response();
                println!("response: {}", resp.status());
                Ok(resp)
            }
            Err(err) => {
                println!("error: {}", err);
                Err(err)
            }
        }
    }
}
