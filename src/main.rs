use rust_tcp_srv::{
    http::{Context, MiddlewareResult, ResponseBuilder},
    logger::LogLevel,
    Config, Logger, Server,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut server = Server::new(Config::default());
    server.static_file("/", "index.html");
    routes(&mut server);
    register_middleware(&mut server);
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
    match serde_json::to_string(&cookies) {
        Ok(json) => ResponseBuilder::ok().json(json).build(),
        Err(_) => ResponseBuilder::server_error()
            .text("Failed to serialize cookies")
            .build(),
    }
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
    let mut api = server.router.group("/api");

    let mut data = api.group("/data");

    data.put("/:id", put_handler).delete("/:id", delete_handler);

    let mut user_group = api.group("/user");

    user_group
        .get("/:id", user_handler)
        .post("/", post_handler)
        .delete("/:id", delete_handler);

    server
        .router
        .get("/api", root_handler)
        .get("/user/:id", user_handler)
        .get("/cookies", cookies_handler)
        .post("/api", post_handler)
        .add_group(data)
        .add_group(user_group);
}

fn global_middleware(ctx: Context) -> MiddlewareResult {
    let logger = Logger::new();
    logger.log(LogLevel::Info, "Global Middleware executed");
    Ok(ctx)
}

fn specific_middleware(ctx: Context) -> MiddlewareResult {
    let logger = Logger::new();
    logger.log(
        LogLevel::Info,
        format!(
            "Specific Middleware executed in route: {}",
            ctx.request.path
        )
        .as_str(),
    );
    Ok(ctx)
}

fn register_middleware(server: &mut Server) {
    server.middleware.add_global(global_middleware);
    server
        .middleware
        .for_route("/api/data/*", specific_middleware);
}
