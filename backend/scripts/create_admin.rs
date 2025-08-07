use collaborative_docs_rs::{
    database::Database,
    models::{SignupRequest, UpdateUserRoleRequest},
    auth::hash_password,
};
use std::env;
use validator::Validate;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: cargo run --bin create_admin <email> <password>");
        std::process::exit(1);
    }

    let email = &args[1];
    let password = &args[2];

    // Get database URL from environment
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://collaborative_user:collaborative_password@localhost:5432/collaborative_docs".to_string());

    println!("Connecting to database...");
    let database = Database::new(&database_url).await?;
    println!("Connected successfully!");

    // Create signup request
    let signup_request = SignupRequest {
        email: email.to_string(),
        password: password.to_string(),
    };

    // Validate input
    signup_request.validate().map_err(|e| {
        format!("Validation failed: {}", e)
    })?;

    // Hash password
    let password_hash = hash_password(&signup_request.password).await?;

    // Create user
    println!("Creating user...");
    let user = database.create_user(&signup_request, &password_hash).await?;
    println!("User created with ID: {}", user.id);

    // Update user to admin role
    println!("Updating user to admin role...");
    let update_request = UpdateUserRoleRequest {
        role_name: "admin".to_string(),
    };

    let updated_user = database.update_user_role(&user.id.to_string(), &update_request.role_name).await?;
    println!("User updated successfully!");
    println!("Email: {}", updated_user.email);
    println!("Role: {}", updated_user.role_name);
    println!("User ID: {}", updated_user.id);

    println!("✅ Admin user created successfully!");
    Ok(())
} 