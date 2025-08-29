use sqlx::{Pool, Postgres};
use validator::{Validate, ValidationError};
// use tokio;

struct TestContext {}
#[derive(Debug, Validate)]
#[validate(context = Db)]
#[validate(schema(function = "validate_claims", skip_on_field_errors = false, use_context))]
struct Claims {
    sub: String,
    iat: String,
    exp: String,
    phone: String,
}


fn validate_iat_and_sub(claims: &Claims, sub: &TestContext) -> Result<(), ValidationError> {
    // true
    Ok(())
}
pub type Db = Pool<Postgres>;

fn validate_claims(claims: &Claims, db: &Db) -> Result<(), ValidationError>  {
    // 1. 检查 sub
    // let user_id = claims.sub.as_str();
    // let user = tokio::runtime::Handle::try_current()?.block_on(async {
    //     sqlx::query_as!(UserTable, "SELECT * FROM user_table WHERE user_id = $1", Uuid::from_str(user_id).unwrap()).fetch_one(&db).await.unwrap()
    // });
    // // 2. 检查phone

    // 3. 检查 iat 和 exp

    Ok(())
}


fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {

    #[test]
    fn validate_claims() {
        println!("Hello, world!");

    }
}