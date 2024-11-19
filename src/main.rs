use rust_tcp_srv::{
    http::{Context, ResponseBuilder},
    logger::LogLevel,
    Config, Logger, Server,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut server = Server::new(Config::default());
    routes(&mut server);
    server.run().await
}

#[derive(Debug, serde::Deserialize)]
pub struct JsonData {
    message: String,
}

fn root_handler(_ctx: &Context) -> Vec<u8> {
    ResponseBuilder::ok_response("Hello from Dean's server!")
}

fn user_handler(ctx: &Context) -> Vec<u8> {
    let user_id = ctx.param("id").unwrap_or("0");
    Logger::new().log(LogLevel::Debug, &format!("User ID: {}", user_id));
    ResponseBuilder::ok().text(format!("{}", user_id)).build()
}

fn cookies_handler(ctx: &Context) -> Vec<u8> {
    let cookies = ctx.request.cookies();
    ResponseBuilder::ok()
        .json(serde_json::to_string(&cookies).unwrap())
        .build()
}

fn post_handler(ctx: &Context) -> Vec<u8> {
    match ctx.request.json_body::<JsonData>() {
        Some(body) => {
            println!("JSON body: {}", body.message);
            ResponseBuilder::created_response("Hello from Dean's server!")
        }
        None => ResponseBuilder::bad_request().text("Bad Request").build(),
    }
}

fn put_handler(ctx: &Context) -> Vec<u8> {
    let id = ctx.param("id").unwrap_or("0");
    ResponseBuilder::created()
        .text(format!("Updated data for ID: {}", id))
        .build()
}

fn delete_handler(ctx: &Context) -> Vec<u8> {
    let id = ctx.param("id").unwrap_or("0");
    ResponseBuilder::deleted()
        .text(format!("Deleted data for ID: {}", id))
        .build()
}

fn routes(server: &mut Server) {
    server
        .router
        .get("/", root_handler)
        .get("/user/:id", user_handler)
        .get("/cookies", cookies_handler)
        .post("/", post_handler)
        .put("/api/data/:id", put_handler)
        .delete("/api/data/:id", delete_handler);
}
