-- Schema Validation Script

-- Function to check if all required extensions are installed
CREATE OR REPLACE FUNCTION public.check_required_extensions()
RETURNS TABLE (
    extension_name TEXT,
    is_installed BOOLEAN
) AS $$
BEGIN
    RETURN QUERY
    WITH required_extensions AS (
        SELECT unnest(ARRAY[
            'uuid-ossp',
            'pgcrypto',
            'postgis'
        ]) AS name
    )
    SELECT 
        r.name,
        EXISTS (
            SELECT 1 
            FROM pg_extension e 
            WHERE e.extname = r.name
        )
    FROM required_extensions r;
END;
$$ LANGUAGE plpgsql;

-- Function to check if all required schemas exist
CREATE OR REPLACE FUNCTION public.check_required_schemas()
RETURNS TABLE (
    schema_name TEXT,
    exists BOOLEAN
) AS $$
BEGIN
    RETURN QUERY
    WITH required_schemas AS (
        SELECT unnest(ARRAY[
            'auth_schema',
            'user_schema',
            'product_schema',
            'rental_schema',
            'payment_schema',
            'messaging_schema',
            'review_schema',
            'subscription_schema',
            'audit_schema'
        ]) AS name
    )
    SELECT 
        r.name,
        EXISTS (
            SELECT 1 
            FROM information_schema.schemata s 
            WHERE s.schema_name = r.name
        )
    FROM required_schemas r;
END;
$$ LANGUAGE plpgsql;

-- Function to validate foreign key relationships
CREATE OR REPLACE FUNCTION public.validate_foreign_keys()
RETURNS TABLE (
    table_schema TEXT,
    table_name TEXT,
    constraint_name TEXT,
    is_valid BOOLEAN,
    invalid_rows BIGINT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        tc.table_schema,
        tc.table_name,
        tc.constraint_name,
        EXISTS (
            SELECT 1
            FROM information_schema.referential_constraints rc
            WHERE rc.constraint_name = tc.constraint_name
        ) AS is_valid,
        COUNT(*) AS invalid_rows
    FROM information_schema.table_constraints tc
    JOIN information_schema.key_column_usage kcu
        ON tc.constraint_name = kcu.constraint_name
    LEFT JOIN information_schema.referential_constraints rc
        ON tc.constraint_name = rc.constraint_name
    WHERE tc.constraint_type = 'FOREIGN KEY'
    GROUP BY tc.table_schema, tc.table_name, tc.constraint_name;
END;
$$ LANGUAGE plpgsql;

-- Function to check for orphaned records
CREATE OR REPLACE FUNCTION public.check_orphaned_records()
RETURNS TABLE (
    table_schema TEXT,
    table_name TEXT,
    fk_column TEXT,
    orphaned_count BIGINT
) AS $$
DECLARE
    r RECORD;
    query TEXT;
BEGIN
    FOR r IN (
        SELECT
            tc.table_schema,
            tc.table_name,
            kcu.column_name as fk_column,
            ccu.table_schema AS foreign_table_schema,
            ccu.table_name AS foreign_table_name,
            ccu.column_name AS foreign_column
        FROM information_schema.table_constraints tc
        JOIN information_schema.key_column_usage kcu
            ON tc.constraint_name = kcu.constraint_name
        JOIN information_schema.constraint_column_usage ccu
            ON ccu.constraint_name = tc.constraint_name
        WHERE tc.constraint_type = 'FOREIGN KEY'
    ) LOOP
        query := format(
            'SELECT %L, %L, %L, COUNT(*) FROM %I.%I t1 WHERE NOT EXISTS (SELECT 1 FROM %I.%I t2 WHERE t2.%I = t1.%I)',
            r.table_schema,
            r.table_name,
            r.fk_column,
            r.table_schema,
            r.table_name,
            r.foreign_table_schema,
            r.foreign_table_name,
            r.foreign_column,
            r.fk_column
        );
        
        RETURN QUERY EXECUTE query;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Function to check for missing indexes on foreign keys
CREATE OR REPLACE FUNCTION public.check_missing_fk_indexes()
RETURNS TABLE (
    table_schema TEXT,
    table_name TEXT,
    column_name TEXT,
    has_index BOOLEAN
) AS $$
BEGIN
    RETURN QUERY
    WITH fk_columns AS (
        SELECT
            tc.table_schema,
            tc.table_name,
            kcu.column_name
        FROM information_schema.table_constraints tc
        JOIN information_schema.key_column_usage kcu
            ON tc.constraint_name = kcu.constraint_name
        WHERE tc.constraint_type = 'FOREIGN KEY'
    )
    SELECT
        f.table_schema,
        f.table_name,
        f.column_name,
        EXISTS (
            SELECT 1
            FROM pg_index i
            JOIN pg_attribute a ON a.attrelid = i.indrelid
            WHERE a.attname = f.column_name
            AND i.indrelid = (f.table_schema || '.' || f.table_name)::regclass
        ) AS has_index
    FROM fk_columns f;
END;
$$ LANGUAGE plpgsql;

-- Function to check for tables without primary keys
CREATE OR REPLACE FUNCTION public.check_tables_without_pk()
RETURNS TABLE (
    schema_name TEXT,
    table_name TEXT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        t.table_schema,
        t.table_name
    FROM information_schema.tables t
    LEFT JOIN information_schema.table_constraints tc
        ON tc.table_schema = t.table_schema
        AND tc.table_name = t.table_name
        AND tc.constraint_type = 'PRIMARY KEY'
    WHERE t.table_schema NOT IN ('pg_catalog', 'information_schema')
        AND t.table_type = 'BASE TABLE'
        AND tc.constraint_name IS NULL;
END;
$$ LANGUAGE plpgsql;

-- Function to validate audit triggers
CREATE OR REPLACE FUNCTION public.validate_audit_triggers()
RETURNS TABLE (
    schema_name TEXT,
    table_name TEXT,
    has_audit_trigger BOOLEAN
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        schemaname::TEXT,
        tablename::TEXT,
        EXISTS (
            SELECT 1
            FROM pg_trigger t
            WHERE t.tgrelid = (schemaname || '.' || tablename)::regclass
            AND t.tgname LIKE '%audit_trigger'
        ) AS has_audit_trigger
    FROM pg_tables
    WHERE schemaname NOT IN ('pg_catalog', 'information_schema', 'public')
    AND schemaname IN (
        'auth_schema',
        'user_schema',
        'product_schema',
        'rental_schema',
        'payment_schema',
        'messaging_schema',
        'review_schema',
        'subscription_schema'
    );
END;
$$ LANGUAGE plpgsql; 