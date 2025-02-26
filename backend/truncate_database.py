try:
    import psycopg2
except ModuleNotFoundError:
    print("Error: psycopg2 is not installed. Install it using: pip install psycopg2")
    exit(1)

import os
from urllib.parse import urlparse

# Parse the DATABASE_URL
DATABASE_URL = os.getenv('DATABASE_URL', 'postgres://postgres:postgres@localhost:5432/strongeryou')
url = urlparse(DATABASE_URL)

# Extract connection details
dbname = url.path[1:]  # Remove the leading '/'
user = url.username
password = url.password
host = url.hostname
port = url.port

try:
    # Connect to the database
    conn = psycopg2.connect(
        dbname=dbname,
        user=user,
        password=password,
        host=host,
        port=port
    )
    cur = conn.cursor()

    # Get all table names in the 'public' schema
    cur.execute("SELECT tablename FROM pg_tables WHERE schemaname = 'public';")
    tables = cur.fetchall()

    # Truncate each table
    for table in tables:
        table_name = table[0]
        # Quote the table name to handle reserved keywords
        cur.execute(f'TRUNCATE TABLE "{table_name}" CASCADE;')
        print(f"Truncated table: {table_name}")

    # Commit the changes and close the connection
    conn.commit()
    cur.close()
    conn.close()

    print("All tables truncated successfully.")

except psycopg2.Error as e:
    print(f"Database error: {e}")