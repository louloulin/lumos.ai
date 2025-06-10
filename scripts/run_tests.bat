@echo off
REM LumosAI Test Execution Script for Windows
REM Comprehensive test runner for the LumosAI framework

setlocal enabledelayedexpansion

REM Configuration
set COVERAGE_THRESHOLD=80
set PERFORMANCE_BASELINE_SECONDS=300
set REPORT_DIR=target\test-reports
set COVERAGE_DIR=target\coverage

REM Create directories
if not exist "%REPORT_DIR%" mkdir "%REPORT_DIR%"
if not exist "%COVERAGE_DIR%" mkdir "%COVERAGE_DIR%"

REM Function to print status messages
:print_status
echo [INFO] %~1
goto :eof

:print_success
echo [SUCCESS] %~1
goto :eof

:print_warning
echo [WARNING] %~1
goto :eof

:print_error
echo [ERROR] %~1
goto :eof

REM Function to check prerequisites
:check_prerequisites
call :print_status "Checking prerequisites..."

REM Check Rust installation
cargo --version >nul 2>&1
if errorlevel 1 (
    call :print_error "Cargo not found. Please install Rust."
    exit /b 1
)

call :print_success "Prerequisites check passed"
goto :eof

REM Function to run unit tests
:run_unit_tests
call :print_status "Running unit tests..."

set start_time=%time%
cargo test --lib --tests unit
set exit_code=%errorlevel%

if %exit_code% equ 0 (
    call :print_success "Unit tests completed successfully"
) else (
    call :print_error "Unit tests failed"
)

exit /b %exit_code%

REM Function to run integration tests
:run_integration_tests
call :print_status "Running integration tests..."

set start_time=%time%
cargo test --tests integration
set exit_code=%errorlevel%

if %exit_code% equ 0 (
    call :print_success "Integration tests completed successfully"
) else (
    call :print_error "Integration tests failed"
)

exit /b %exit_code%

REM Function to run performance tests
:run_performance_tests
call :print_status "Running performance tests..."

set start_time=%time%
cargo test --tests performance --release
set exit_code=%errorlevel%

if %exit_code% equ 0 (
    call :print_success "Performance tests completed successfully"
) else (
    call :print_error "Performance tests failed"
)

exit /b %exit_code%

REM Function to validate examples
:validate_examples
call :print_status "Validating examples..."

set failed_examples=0
set total_examples=0

REM List of known examples to test
set examples=basic_agent rag_system tool_integration memory_system vector_storage streaming_response multi_agent_workflow enhanced_features_demo performance_benchmark auth_demo monitoring_demo_simple simplified_api_complete_demo

for %%e in (%examples%) do (
    set /a total_examples+=1
    call :print_status "Validating example: %%e"
    
    cargo run --example %%e >nul 2>&1
    if errorlevel 1 (
        call :print_error "Example %%e failed"
        set /a failed_examples+=1
    ) else (
        call :print_success "Example %%e validated"
    )
)

if %failed_examples% equ 0 (
    call :print_success "All %total_examples% examples validated"
    exit /b 0
) else (
    call :print_error "%failed_examples% out of %total_examples% examples failed"
    exit /b 1
)

REM Function to generate coverage report
:generate_coverage
call :print_status "Generating coverage report..."

REM Check if tarpaulin is available (may not work on Windows)
cargo tarpaulin --version >nul 2>&1
if errorlevel 1 (
    call :print_warning "cargo-tarpaulin not found or not supported on Windows"
    call :print_status "Consider using alternative coverage tools or running on Linux/macOS"
    goto :eof
)

cargo tarpaulin --out Html --output-dir "%COVERAGE_DIR%" --timeout 600
if errorlevel 1 (
    call :print_error "Coverage generation failed"
    exit /b 1
) else (
    call :print_success "Coverage report generated in %COVERAGE_DIR%"
)

goto :eof

REM Function to run quality checks
:run_quality_checks
call :print_status "Running code quality checks..."

REM Check formatting
call :print_status "Checking code formatting..."
cargo fmt --check
if errorlevel 1 (
    call :print_error "Code formatting check failed. Run 'cargo fmt' to fix."
    exit /b 1
) else (
    call :print_success "Code formatting check passed"
)

REM Run clippy
call :print_status "Running clippy lints..."
cargo clippy --all-targets --all-features -- -D warnings
if errorlevel 1 (
    call :print_error "Clippy check failed"
    exit /b 1
) else (
    call :print_success "Clippy check passed"
)

exit /b 0

REM Function to generate test report
:generate_report
set total_tests=%1
set passed_tests=%2

set /a failed_tests=%total_tests%-%passed_tests%

for /f "tokens=2 delims==" %%a in ('wmic OS Get localdatetime /value') do set "dt=%%a"
set "timestamp=%dt:~0,4%-%dt:~4,2%-%dt:~6,2%_%dt:~8,2%%dt:~10,2%%dt:~12,2%"

set report_file=%REPORT_DIR%\test_report_%timestamp%.md

echo # LumosAI Test Report > "%report_file%"
echo. >> "%report_file%"
echo **Generated:** %date% %time% >> "%report_file%"
echo. >> "%report_file%"
echo ## Summary >> "%report_file%"
echo. >> "%report_file%"
echo - **Total Test Suites:** %total_tests% >> "%report_file%"
echo - **Passed:** %passed_tests% ✅ >> "%report_file%"
echo - **Failed:** %failed_tests% ❌ >> "%report_file%"
echo. >> "%report_file%"

if %failed_tests% equ 0 (
    echo 🎉 **ALL TESTS PASSED!** >> "%report_file%"
) else (
    echo ⚠️ **%failed_tests% test suite(s) failed** >> "%report_file%"
)

call :print_success "Test report generated: %report_file%"
goto :eof

REM Main execution function
:main
call :print_status "🚀 Starting LumosAI test suite execution..."
echo ==================================================

set total_tests=0
set passed_tests=0

REM Check prerequisites
call :check_prerequisites
if errorlevel 1 exit /b 1

REM Run quality checks first
set /a total_tests+=1
call :run_quality_checks
if not errorlevel 1 set /a passed_tests+=1

REM Run unit tests
set /a total_tests+=1
call :run_unit_tests
if not errorlevel 1 set /a passed_tests+=1

REM Run integration tests
set /a total_tests+=1
call :run_integration_tests
if not errorlevel 1 set /a passed_tests+=1

REM Validate examples
set /a total_tests+=1
call :validate_examples
if not errorlevel 1 set /a passed_tests+=1

REM Run performance tests
set /a total_tests+=1
call :run_performance_tests
if not errorlevel 1 set /a passed_tests+=1

REM Generate coverage report
call :generate_coverage

REM Generate final report
call :generate_report %total_tests% %passed_tests%

REM Print final summary
echo.
echo ==================================================
call :print_status "🏁 Test execution completed"

set /a failed_tests=%total_tests%-%passed_tests%

echo 📊 Final Summary:
echo    Total test suites: %total_tests%
echo    Passed: %passed_tests% ✅
echo    Failed: %failed_tests% ❌

if %failed_tests% equ 0 (
    call :print_success "🎉 ALL TESTS PASSED!"
    exit /b 0
) else (
    call :print_error "⚠️ %failed_tests% test suite(s) failed"
    exit /b 1
)

REM Handle command line arguments
if "%1"=="unit" (
    call :check_prerequisites
    call :run_unit_tests
    exit /b %errorlevel%
)

if "%1"=="integration" (
    call :check_prerequisites
    call :run_integration_tests
    exit /b %errorlevel%
)

if "%1"=="performance" (
    call :check_prerequisites
    call :run_performance_tests
    exit /b %errorlevel%
)

if "%1"=="examples" (
    call :check_prerequisites
    call :validate_examples
    exit /b %errorlevel%
)

if "%1"=="coverage" (
    call :check_prerequisites
    call :generate_coverage
    exit /b %errorlevel%
)

if "%1"=="quality" (
    call :check_prerequisites
    call :run_quality_checks
    exit /b %errorlevel%
)

REM Default: run all tests
call :main
exit /b %errorlevel%
