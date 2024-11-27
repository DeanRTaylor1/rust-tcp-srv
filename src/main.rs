use rust_tcp_srv::{
    http::{Context, MiddlewareResult, ResponseBuilder},
    logger::LogLevel,
    Config, Logger, Server,
};
use std::marker::PhantomData;
use std::collections::{HashSet, HashMap};



// SQL type system that maps to database column types
#[derive(Debug, Clone)]
enum SqlType {
    Text,
    Integer,
    Float,
    Boolean,
}

impl SqlType {
    fn to_string(&self) -> &'static str {
        match self {
            SqlType::Text => "TEXT",
            SqlType::Integer => "INTEGER",
            SqlType::Float => "FLOAT",
            SqlType::Boolean => "BOOLEAN",
        }
    }
}

// Represents a database column with its name and type
type Column = (&'static str, SqlType);

// Core trait that all database tables must implement
trait Table {
    fn primary_key(&self) -> i32;
    fn table_name() -> &'static str;
    type Joins: JoinTypes;
    fn columns() -> Vec<Column>;
}

// Defines the possible joins for a table
trait JoinTypes {
    type Joinable;
}

// Defines how tables can be joined together
trait Joinable {
    type Target: Table;
    fn foreign_key(&self) -> i32;
}

// Types of WHERE clauses we support
#[derive(Debug)]
enum WhereClause {
    And(String),
    Or(String),
}

struct NoJoins;

// Changed to use a more specific type for joins
struct WithJoin<Base, Joined>(PhantomData<(Base, Joined)>);

// Implement JoinTypes for the unit type to handle tables with no joins
impl JoinTypes for () {
    type Joinable = ();
}

// Now we implement Contains with more specific rules
trait Contains<T> {}

// Base case: QueryBuilder contains its base table
impl<Base: Table, Joins> Contains<Base> for QueryBuilder<Base, Joins> {}

// Direct join case: A join state contains the table it's joining to
impl<Base, Joined> Contains<Joined> for WithJoin<Base, Joined> {}

// Base case for NoJoins
impl<T: Table> Contains<T> for NoJoins where T: Table {}

// Main query builder with type-level tracking of base table and joins
#[derive(Debug)]
struct QueryBuilder<Base: Table, Joins = NoJoins> {
    select: Vec<String>,
    from: String,
    joins: HashSet<String>,
    where_clauses: Vec<WhereClause>,
    available_columns: HashMap<String, Vec<Column>>,
    _phantom: PhantomData<(Base, Joins)>,
}

// Implementation for creating new query builders
impl<Base: Table> QueryBuilder<Base, NoJoins> {
    fn new() -> Self {
        let mut available_columns = HashMap::new();
        available_columns.insert(Base::table_name().to_string(), Base::columns());

        Self {
            select: Base::columns().iter()
                .map(|(name, _)| format!("{}.{}", Base::table_name(), name))
                .collect(),
            from: Base::table_name().to_string(),
            joins: HashSet::new(),
            where_clauses: vec![],
            available_columns,
            _phantom: PhantomData,
        }
    }
}

impl<Base: Table, CurrentJoins> QueryBuilder<Base, CurrentJoins> {
    // Join now creates a WithJoin type that chains the joins together
    fn join<T: Table>(mut self) -> QueryBuilder<Base, WithJoin<CurrentJoins, T>> {
        self.joins.insert(T::table_name().to_string());
        self.available_columns.insert(T::table_name().to_string(), T::columns());

        QueryBuilder {
            select: self.select,
            from: self.from,
            joins: self.joins,
            where_clauses: self.where_clauses,
            available_columns: self.available_columns,
            _phantom: PhantomData,
        }
    }

    fn and_where<T>(mut self, column: &str, value: &str) -> Self
    where
        CurrentJoins: Contains<T>,
        T: Table,
    {
        let qualified_column = if column.contains('.') {
            column.to_string()
        } else {
            format!("{}.{}", T::table_name(), column)
        };

        self.where_clauses.push(WhereClause::And(
            format!("{} = '{}'", qualified_column, value)
        ));
        self
    }

    // Add OR clause - only allowed if table is joined or is base table
    fn or_where<T>(mut self, column: &str, value: &str) -> Self
    where
        Self: Contains<T>,
        T: Table,
    {
        let qualified_column = if column.contains('.') {
            column.to_string()
        } else {
            format!("{}.{}", T::table_name(), column)
        };

        self.where_clauses.push(WhereClause::Or(
            format!("{} = '{}'", qualified_column, value)
        ));
        self
    }

    // Generate the final SQL string
    fn to_sql(self) -> String {
        let mut query = format!("SELECT {} FROM {}", self.select.join(", "), self.from);

        // Add joins
        if !self.joins.is_empty() {
            query.push_str(" ");
            for joined_table in self.joins {
                query.push_str(&format!(
                    "JOIN {} ON {}.{}_id = {}.id ",
                    joined_table,
                    self.from,
                    joined_table.trim_end_matches('s'),
                    joined_table
                ));
            }
        }

        // Add where clauses
        if !self.where_clauses.is_empty() {
            query.push_str("WHERE ");
            let clauses: Vec<String> = self.where_clauses
                .iter()
                .enumerate()
                .map(|(i, clause)| match clause {
                    WhereClause::And(c) => {
                        if i == 0 { c.clone() } else { format!("AND {}", c) }
                    },
                    WhereClause::Or(c) => format!("OR {}", c),
                })
                .collect();
            query.push_str(&clauses.join(" "));
        }

        query
    }
}


// Example table definitions
#[derive(Debug)]
struct User {
    id: i32,
    profile_id: i32,
}

#[derive(Debug)]
struct Profile {
    id: i32,
}

// Define available joins for User table
enum UserJoins {
    Profile,
}

impl JoinTypes for UserJoins {
    type Joinable = User;
}

// Implement Table trait for User
impl Table for User {
    fn primary_key(&self) -> i32 { self.id }
    fn table_name() -> &'static str { "users" }
    type Joins = UserJoins;
    fn columns() -> Vec<Column> {
        vec![
            ("id", SqlType::Integer),
            ("profile_id", SqlType::Integer),
        ]
    }
}

// Implement Table trait for Profile
impl Table for Profile {
    fn primary_key(&self) -> i32 { self.id }
    fn table_name() -> &'static str { "profiles" }
    type Joins = ();
    fn columns() -> Vec<Column> {
        vec![
            ("id", SqlType::Integer),
        ]
    }
}



#[tokio::main]
async fn main() -> std::io::Result<()> {

    // Example usage
    let query = QueryBuilder::<User>::new()
        .and_where::<User>("id", "1")  // This works now!
        .join::<Profile>()
        .to_sql();    println!("{}", query);

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
