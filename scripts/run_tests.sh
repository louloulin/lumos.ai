#!/bin/bash

# LumosAI Test Execution Script
# Comprehensive test runner for the LumosAI framework

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
COVERAGE_THRESHOLD=80
PERFORMANCE_BASELINE_SECONDS=300
REPORT_DIR="target/test-reports"
COVERAGE_DIR="target/coverage"

# Create directories
mkdir -p "$REPORT_DIR"
mkdir -p "$COVERAGE_DIR"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to run tests with timeout
run_with_timeout() {
    local timeout=$1
    local command=$2
    local description=$3
    
    print_status "Running $description (timeout: ${timeout}s)..."
    
    if timeout "$timeout" bash -c "$command"; then
        print_success "$description completed successfully"
        return 0
    else
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            print_error "$description timed out after ${timeout}s"
        else
            print_error "$description failed with exit code $exit_code"
        fi
        return $exit_code
    fi
}

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    # Check Rust installation
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Rust."
        exit 1
    fi
    
    # Check required tools
    if ! command -v timeout &> /dev/null; then
        print_warning "timeout command not found. Tests may not respect timeouts."
    fi
    
    print_success "Prerequisites check passed"
}

# Function to run unit tests
run_unit_tests() {
    print_status "Running unit tests..."
    
    local start_time=$(date +%s)
    
    if run_with_timeout 300 "cargo test --lib --tests unit" "Unit tests"; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        print_success "Unit tests completed in ${duration}s"
        return 0
    else
        print_error "Unit tests failed"
        return 1
    fi
}

# Function to run integration tests
run_integration_tests() {
    print_status "Running integration tests..."
    
    local start_time=$(date +%s)
    
    if run_with_timeout 600 "cargo test --tests integration" "Integration tests"; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        print_success "Integration tests completed in ${duration}s"
        return 0
    else
        print_error "Integration tests failed"
        return 1
    fi
}

# Function to run performance tests
run_performance_tests() {
    print_status "Running performance tests..."
    
    local start_time=$(date +%s)
    
    if run_with_timeout 1800 "cargo test --tests performance --release" "Performance tests"; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        
        if [ $duration -le $PERFORMANCE_BASELINE_SECONDS ]; then
            print_success "Performance tests completed in ${duration}s (within baseline of ${PERFORMANCE_BASELINE_SECONDS}s)"
        else
            print_warning "Performance tests completed in ${duration}s (exceeds baseline of ${PERFORMANCE_BASELINE_SECONDS}s)"
        fi
        return 0
    else
        print_error "Performance tests failed"
        return 1
    fi
}

# Function to validate examples
validate_examples() {
    print_status "Validating examples..."
    
    local start_time=$(date +%s)
    local failed_examples=0
    local total_examples=0
    
    # Get list of examples
    local examples=$(find examples -name "*.rs" -type f | head -12)  # Limit to first 12 examples
    
    for example in $examples; do
        local example_name=$(basename "$example" .rs)
        total_examples=$((total_examples + 1))
        
        print_status "Validating example: $example_name"
        
        if timeout 60 cargo run --example "$example_name" > /dev/null 2>&1; then
            print_success "Example $example_name validated"
        else
            print_error "Example $example_name failed"
            failed_examples=$((failed_examples + 1))
        fi
    done
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    if [ $failed_examples -eq 0 ]; then
        print_success "All $total_examples examples validated in ${duration}s"
        return 0
    else
        print_error "$failed_examples out of $total_examples examples failed"
        return 1
    fi
}

# Function to generate coverage report
generate_coverage() {
    print_status "Generating coverage report..."
    
    # Check if tarpaulin is installed
    if command -v cargo-tarpaulin &> /dev/null; then
        print_status "Using cargo-tarpaulin for coverage..."
        
        if cargo tarpaulin --out Html --output-dir "$COVERAGE_DIR" --timeout 600; then
            print_success "Coverage report generated in $COVERAGE_DIR"
            
            # Extract coverage percentage
            if [ -f "$COVERAGE_DIR/tarpaulin-report.html" ]; then
                local coverage=$(grep -o '[0-9]*\.[0-9]*%' "$COVERAGE_DIR/tarpaulin-report.html" | head -1 | sed 's/%//')
                
                if [ -n "$coverage" ]; then
                    local coverage_int=$(echo "$coverage" | cut -d'.' -f1)
                    
                    if [ "$coverage_int" -ge "$COVERAGE_THRESHOLD" ]; then
                        print_success "Coverage: ${coverage}% (meets threshold of ${COVERAGE_THRESHOLD}%)"
                    else
                        print_warning "Coverage: ${coverage}% (below threshold of ${COVERAGE_THRESHOLD}%)"
                    fi
                fi
            fi
        else
            print_error "Coverage generation failed"
            return 1
        fi
    else
        print_warning "cargo-tarpaulin not found. Skipping coverage report."
        print_status "Install with: cargo install cargo-tarpaulin"
    fi
}

# Function to run linting and formatting checks
run_quality_checks() {
    print_status "Running code quality checks..."
    
    # Check formatting
    print_status "Checking code formatting..."
    if cargo fmt --check; then
        print_success "Code formatting check passed"
    else
        print_error "Code formatting check failed. Run 'cargo fmt' to fix."
        return 1
    fi
    
    # Run clippy
    print_status "Running clippy lints..."
    if cargo clippy --all-targets --all-features -- -D warnings; then
        print_success "Clippy check passed"
    else
        print_error "Clippy check failed"
        return 1
    fi
    
    return 0
}

# Function to generate test report
generate_report() {
    local total_tests=$1
    local passed_tests=$2
    local start_time=$3
    local end_time=$4
    
    local duration=$((end_time - start_time))
    local failed_tests=$((total_tests - passed_tests))
    
    local report_file="$REPORT_DIR/test_report_$(date +%Y%m%d_%H%M%S).md"
    
    cat > "$report_file" << EOF
# LumosAI Test Report

**Generated:** $(date)

## Summary

- **Total Test Suites:** $total_tests
- **Passed:** $passed_tests ✅
- **Failed:** $failed_tests ❌
- **Success Rate:** $(( (passed_tests * 100) / total_tests ))%
- **Total Duration:** ${duration}s

## Test Results

EOF

    if [ $failed_tests -eq 0 ]; then
        echo "🎉 **ALL TESTS PASSED!**" >> "$report_file"
    else
        echo "⚠️ **$failed_tests test suite(s) failed**" >> "$report_file"
    fi
    
    echo "" >> "$report_file"
    echo "## Coverage" >> "$report_file"
    
    if [ -f "$COVERAGE_DIR/tarpaulin-report.html" ]; then
        echo "Coverage report available at: \`$COVERAGE_DIR/tarpaulin-report.html\`" >> "$report_file"
    else
        echo "Coverage report not generated" >> "$report_file"
    fi
    
    print_success "Test report generated: $report_file"
}

# Main execution function
main() {
    local start_time=$(date +%s)
    local total_tests=0
    local passed_tests=0
    
    print_status "🚀 Starting LumosAI test suite execution..."
    echo "=================================================="
    
    # Check prerequisites
    check_prerequisites
    
    # Run quality checks first
    total_tests=$((total_tests + 1))
    if run_quality_checks; then
        passed_tests=$((passed_tests + 1))
    fi
    
    # Run unit tests
    total_tests=$((total_tests + 1))
    if run_unit_tests; then
        passed_tests=$((passed_tests + 1))
    fi
    
    # Run integration tests
    total_tests=$((total_tests + 1))
    if run_integration_tests; then
        passed_tests=$((passed_tests + 1))
    fi
    
    # Validate examples
    total_tests=$((total_tests + 1))
    if validate_examples; then
        passed_tests=$((passed_tests + 1))
    fi
    
    # Run performance tests
    total_tests=$((total_tests + 1))
    if run_performance_tests; then
        passed_tests=$((passed_tests + 1))
    fi
    
    # Generate coverage report
    generate_coverage
    
    local end_time=$(date +%s)
    
    # Generate final report
    generate_report $total_tests $passed_tests $start_time $end_time
    
    # Print final summary
    echo ""
    echo "=================================================="
    print_status "🏁 Test execution completed"
    
    local duration=$((end_time - start_time))
    local failed_tests=$((total_tests - passed_tests))
    
    echo "📊 Final Summary:"
    echo "   Total test suites: $total_tests"
    echo "   Passed: $passed_tests ✅"
    echo "   Failed: $failed_tests ❌"
    echo "   Total time: ${duration}s"
    
    if [ $failed_tests -eq 0 ]; then
        print_success "🎉 ALL TESTS PASSED!"
        exit 0
    else
        print_error "⚠️ $failed_tests test suite(s) failed"
        exit 1
    fi
}

# Handle command line arguments
case "${1:-all}" in
    "unit")
        check_prerequisites
        run_unit_tests
        ;;
    "integration")
        check_prerequisites
        run_integration_tests
        ;;
    "performance")
        check_prerequisites
        run_performance_tests
        ;;
    "examples")
        check_prerequisites
        validate_examples
        ;;
    "coverage")
        check_prerequisites
        generate_coverage
        ;;
    "quality")
        check_prerequisites
        run_quality_checks
        ;;
    "all"|*)
        main
        ;;
esac
