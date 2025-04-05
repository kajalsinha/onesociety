#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script location and paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DB_DIR="$(dirname "$SCRIPT_DIR")"
SQL_DIR="$DB_DIR/sql"
VALIDATION_SQL="$SQL_DIR/validate_schema.sql"

# Default database URL (can be overridden by environment variable)
DATABASE_URL=${DATABASE_URL:-"postgresql://onesociety:development_password@localhost:5432/onesociety"}

# Functions to print headers and check results
print_section() {
    echo -e "\n${BLUE}======== $1 ========${NC}"
}

check_result() {
    if [ "$1" -eq 0 ]; then
        echo -e "${GREEN}✓ Pass${NC}"
        return 0
    else
        echo -e "${RED}✗ Failed${NC}"
        return 1
    fi
}

# Initialize validation functions
initialize_validation() {
    print_section "Initializing schema validation functions"
    
    # First ensure the validation functions exist
    if ! psql "$DATABASE_URL" -f "$VALIDATION_SQL" > /dev/null; then
        echo -e "${RED}Error: Failed to initialize schema validation functions.${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}Validation functions initialized.${NC}"
}

# Run all validations and return non-zero if any fail
validate_all() {
    local failures=0
    
    # First initialize the validation functions
    initialize_validation
    
    # Check required extensions
    print_section "Checking required extensions"
    if ! psql "$DATABASE_URL" -c "SELECT * FROM check_required_extensions()" -t | grep -q "t"; then
        echo -e "${RED}✗ Required extensions check failed${NC}"
        psql "$DATABASE_URL" -c "SELECT * FROM check_required_extensions()"
        failures=$((failures + 1))
    else
        echo -e "${GREEN}✓ All required extensions installed${NC}"
    fi
    
    # Check required schemas
    print_section "Checking required schemas"
    if ! psql "$DATABASE_URL" -c "SELECT * FROM check_required_schemas()" -t | grep -q "t"; then
        echo -e "${RED}✗ Required schemas check failed${NC}"
        psql "$DATABASE_URL" -c "SELECT * FROM check_required_schemas()"
        failures=$((failures + 1))
    else
        echo -e "${GREEN}✓ All required schemas exist${NC}"
    fi
    
    # Check foreign keys
    print_section "Validating foreign key relationships"
    if ! psql "$DATABASE_URL" -c "SELECT * FROM validate_foreign_keys()" -t | grep -q "t"; then
        echo -e "${RED}✗ Foreign key validation failed${NC}"
        failures=$((failures + 1))
    else
        echo -e "${GREEN}✓ All foreign key relationships are valid${NC}"
    fi
    
    # Check for orphaned records
    print_section "Checking for orphaned records"
    if ! psql "$DATABASE_URL" -c "SELECT * FROM check_orphaned_records()" -t | grep -q "t"; then
        echo -e "${RED}✗ Orphaned records found${NC}"
        failures=$((failures + 1))
    else
        echo -e "${GREEN}✓ No orphaned records detected${NC}"
    fi
    
    # Check for missing FK indexes
    print_section "Checking for missing foreign key indexes"
    if ! psql "$DATABASE_URL" -c "SELECT * FROM check_missing_fk_indexes()" -t | grep -q "t"; then
        echo -e "${RED}✗ Missing foreign key indexes found${NC}"
        failures=$((failures + 1))
    else
        echo -e "${GREEN}✓ All foreign keys have appropriate indexes${NC}"
    fi
    
    # Check for tables without primary keys
    print_section "Checking for tables without primary keys"
    if ! psql "$DATABASE_URL" -c "SELECT * FROM check_tables_without_pk()" -t | grep -q "t"; then
        echo -e "${RED}✗ Tables without primary keys found${NC}"
        failures=$((failures + 1))
    else
        echo -e "${GREEN}✓ All tables have primary keys${NC}"
    fi
    
    # Check audit triggers
    print_section "Validating audit triggers"
    if ! psql "$DATABASE_URL" -c "SELECT * FROM validate_audit_triggers()" -t | grep -q "t"; then
        echo -e "${RED}✗ Audit trigger validation failed${NC}"
        failures=$((failures + 1))
    else
        echo -e "${GREEN}✓ All required audit triggers are configured${NC}"
    fi
    
    # Summary
    print_section "Validation Summary"
    if [ $failures -eq 0 ]; then
        echo -e "${GREEN}All schema validations passed.${NC}"
        echo -e "\nFor detailed reports, run:"
        echo -e "${BLUE}> make db-validate-full${NC}"
    else
        echo -e "${RED}$failures validation checks failed.${NC}"
        echo -e "\nFor detailed reports on failures, run:"
        echo -e "${BLUE}> psql \$DATABASE_URL -c \"SELECT * FROM check_required_extensions()\"${NC}"
        echo -e "${BLUE}> psql \$DATABASE_URL -c \"SELECT * FROM check_required_schemas()\"${NC}"
        echo -e "${BLUE}> psql \$DATABASE_URL -c \"SELECT * FROM validate_foreign_keys()\"${NC}"
        echo -e "${BLUE}> psql \$DATABASE_URL -c \"SELECT * FROM check_orphaned_records()\"${NC}"
        echo -e "${BLUE}> psql \$DATABASE_URL -c \"SELECT * FROM check_missing_fk_indexes()\"${NC}"
        echo -e "${BLUE}> psql \$DATABASE_URL -c \"SELECT * FROM check_tables_without_pk()\"${NC}"
        echo -e "${BLUE}> psql \$DATABASE_URL -c \"SELECT * FROM validate_audit_triggers()\"${NC}"
    fi
    
    return $failures
}

# Main execution
validate_all
exit $? 