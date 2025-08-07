# Database Troubleshooting Guide

This guide helps resolve common database permission and connection issues.

## 🔧 Permission Denied for Schema Public

### Problem
```
error: while executing migrations: error returned from database: permission denied for schema public
```

### Solution

#### Option 1: Fix Existing Database (Recommended)

Connect to PostgreSQL and fix the permissions:

```bash
# Connect as postgres superuser
sudo -u postgres psql

# Connect to your database
\c collaborative_docs

# Grant proper permissions to your user
GRANT ALL PRIVILEGES ON SCHEMA public TO collaborative_user;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO collaborative_user;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO collaborative_user;
GRANT ALL PRIVILEGES ON ALL FUNCTIONS IN SCHEMA public TO collaborative_user;

-- Grant privileges on future objects
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL PRIVILEGES ON TABLES TO collaborative_user;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL PRIVILEGES ON SEQUENCES TO collaborative_user;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL PRIVILEGES ON FUNCTIONS TO collaborative_user;

-- Exit
\q
```

#### Option 2: Recreate Database with Proper Permissions

```bash
# Drop and recreate the database
sudo -u postgres psql << EOF
DROP DATABASE IF EXISTS collaborative_docs;
DROP USER IF EXISTS collaborative_user;

CREATE DATABASE collaborative_docs;
CREATE USER collaborative_user WITH PASSWORD 'collaborative_password_bismillah786';
GRANT ALL PRIVILEGES ON DATABASE collaborative_docs TO collaborative_user;
ALTER USER collaborative_user CREATEDB;
\q
EOF

# Connect as the new user and set up schema permissions
PGPASSWORD=collaborative_password_bismillah786 psql -h localhost -U collaborative_user -d collaborative_docs << EOF
GRANT ALL PRIVILEGES ON SCHEMA public TO collaborative_user;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO collaborative_user;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO collaborative_user;
GRANT ALL PRIVILEGES ON ALL FUNCTIONS IN SCHEMA public TO collaborative_user;

ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL PRIVILEGES ON TABLES TO collaborative_user;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL PRIVILEGES ON SEQUENCES TO collaborative_user;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL PRIVILEGES ON FUNCTIONS TO collaborative_user;
\q
EOF
```

#### Option 3: Use the Automated Setup Script

```bash
# Run the setup script with your custom password
./scripts/setup-database.sh \
  --db-user collaborative_user \
  --db-password collaborative_password_bismillah786
```

### Test the Fix

After applying the fix, test the connection:

```bash
# Test database connection
PGPASSWORD=collaborative_password_bismillah786 psql -h localhost -U collaborative_user -d collaborative_docs -c "SELECT version();"

# Run migrations
export DATABASE_URL="postgresql://collaborative_user:collaborative_password_bismillah786@localhost:5432/collaborative_docs"
sqlx migrate run
```

## 🔍 Other Common Issues

### Connection Refused
```
error: error connecting to the database: connection refused
```

**Solution:**
```bash
# Check if PostgreSQL is running
sudo systemctl status postgresql

# Start PostgreSQL if not running
sudo systemctl start postgresql
```

### Authentication Failed
```
error: error connecting to the database: authentication failed
```

**Solution:**
```bash
# Check if user exists and password is correct
sudo -u postgres psql -c "\du"

# Reset password if needed
sudo -u postgres psql -c "ALTER USER collaborative_user WITH PASSWORD 'new_password';"
```

### Database Does Not Exist
```
error: error connecting to the database: database "collaborative_docs" does not exist
```

**Solution:**
```bash
# Create the database
sudo -u postgres psql -c "CREATE DATABASE collaborative_docs;"
```

## 📋 Quick Fix Commands

### For Your Specific Case

```bash
# Fix permissions for your existing setup
sudo -u postgres psql << 'EOF'
\c collaborative_docs
GRANT ALL PRIVILEGES ON SCHEMA public TO collaborative_user;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO collaborative_user;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO collaborative_user;
GRANT ALL PRIVILEGES ON ALL FUNCTIONS IN SCHEMA public TO collaborative_user;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL PRIVILEGES ON TABLES TO collaborative_user;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL PRIVILEGES ON SEQUENCES TO collaborative_user;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL PRIVILEGES ON FUNCTIONS TO collaborative_user;
\q
EOF

# Test and run migrations
export DATABASE_URL="postgresql://collaborative_user:collaborative_password_bismillah786@localhost:5432/collaborative_docs"
sqlx migrate run
```

### Verify Permissions

```bash
# Check user permissions
sudo -u postgres psql -c "\du collaborative_user"

# Check database permissions
sudo -u postgres psql -c "\l collaborative_docs"

# Test user connection
PGPASSWORD=collaborative_password_bismillah786 psql -h localhost -U collaborative_user -d collaborative_docs -c "SELECT current_user, current_database();"
```

## 🚀 Prevention

To avoid permission issues in the future:

1. **Use the automated setup script**: `./scripts/setup-database.sh`
2. **Follow the manual setup instructions** in `DEPLOYMENT.md`
3. **Always grant schema permissions** after creating the database
4. **Test connections** before running migrations

## 📞 Getting Help

If you're still having issues:

1. Check the PostgreSQL logs: `sudo tail -f /var/log/postgresql/postgresql-*.log`
2. Verify your connection string: `echo $DATABASE_URL`
3. Test with psql: `psql $DATABASE_URL`
4. Check the [PostgreSQL documentation](https://www.postgresql.org/docs/) for your version 