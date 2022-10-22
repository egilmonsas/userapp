use sqlx::FromRow;

#[derive(FromRow)]
pub struct BoolWrapper(pub bool);
