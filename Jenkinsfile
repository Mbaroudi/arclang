pipeline {
    agent any
    
    environment {
        CARGO_HOME = "${WORKSPACE}/.cargo"
        RUST_BACKTRACE = '1'
    }
    
    options {
        buildDiscarder(logRotator(numToKeepStr: '10'))
        timestamps()
        timeout(time: 1, unit: 'HOURS')
    }
    
    stages {
        stage('Setup') {
            steps {
                sh 'rustc --version'
                sh 'cargo --version'
            }
        }
        
        stage('Build') {
            parallel {
                stage('Debug Build') {
                    steps {
                        sh 'cargo build --workspace --verbose'
                    }
                }
                
                stage('Release Build') {
                    steps {
                        sh 'cargo build --release --workspace'
                    }
                }
            }
        }
        
        stage('Test') {
            parallel {
                stage('Unit Tests') {
                    steps {
                        sh 'cargo test --all-features --workspace'
                    }
                }
                
                stage('Doc Tests') {
                    steps {
                        sh 'cargo test --doc --workspace'
                    }
                }
                
                stage('Integration Tests') {
                    steps {
                        script {
                            sh '''
                                ./target/release/arclang check examples/aerospace/flight_control_system.arc
                                ./target/release/arclang check examples/automotive/adaptive_cruise_control.arc
                                ./target/release/arclang check examples/defense/mission_computer.arc
                            '''
                        }
                    }
                }
            }
        }
        
        stage('Quality') {
            parallel {
                stage('Lint') {
                    steps {
                        sh 'rustup component add rustfmt clippy'
                        sh 'cargo fmt --all -- --check'
                        sh 'cargo clippy --all-targets --all-features -- -D warnings'
                    }
                }
                
                stage('Coverage') {
                    steps {
                        sh 'cargo install cargo-tarpaulin || true'
                        sh 'cargo tarpaulin --all-features --workspace --timeout 300 --out Xml'
                        publishCoverage adapters: [coberturaAdapter('cobertura.xml')]
                    }
                }
                
                stage('Security Audit') {
                    steps {
                        sh 'cargo install cargo-audit || true'
                        sh 'cargo audit'
                    }
                }
            }
        }
        
        stage('Safety Analysis') {
            parallel {
                stage('DO-178C Analysis') {
                    steps {
                        sh './target/release/arclang safety examples/aerospace/flight_control_system.arc --standard DO178C --report'
                        archiveArtifacts artifacts: '*_safety_report.pdf', fingerprint: true
                    }
                }
                
                stage('ISO 26262 Analysis') {
                    steps {
                        sh './target/release/arclang safety examples/automotive/adaptive_cruise_control.arc --standard ISO26262 --fmea --fta --report'
                        archiveArtifacts artifacts: '*_fmea.csv,*_fta.dot,*_safety_report.pdf', fingerprint: true
                    }
                }
            }
        }
        
        stage('Benchmarks') {
            when {
                branch 'main'
            }
            steps {
                sh 'cargo bench --workspace'
                archiveArtifacts artifacts: 'target/criterion/**/*', fingerprint: true
            }
        }
        
        stage('Documentation') {
            steps {
                sh 'cargo doc --all-features --no-deps --workspace'
                publishHTML([
                    reportDir: 'target/doc',
                    reportFiles: 'index.html',
                    reportName: 'API Documentation'
                ])
            }
        }
        
        stage('Package') {
            when {
                anyOf {
                    branch 'main'
                    tag pattern: 'v*.*.*', comparator: 'REGEXP'
                }
            }
            steps {
                script {
                    if (env.TAG_NAME) {
                        sh 'cargo package --workspace'
                        archiveArtifacts artifacts: 'target/package/*.crate', fingerprint: true
                    }
                }
            }
        }
        
        stage('Deploy') {
            when {
                tag pattern: 'v*.*.*', comparator: 'REGEXP'
            }
            steps {
                script {
                    withCredentials([string(credentialsId: 'cargo-token', variable: 'CARGO_TOKEN')]) {
                        sh 'cargo publish --token ${CARGO_TOKEN}'
                    }
                }
            }
        }
    }
    
    post {
        always {
            junit testResults: 'target/junit.xml', allowEmptyResults: true
        }
        
        success {
            echo 'Pipeline succeeded!'
        }
        
        failure {
            echo 'Pipeline failed!'
            emailext (
                subject: "Build Failed: ${env.JOB_NAME} - ${env.BUILD_NUMBER}",
                body: "Check console output at ${env.BUILD_URL}",
                to: "${env.CHANGE_AUTHOR_EMAIL}"
            )
        }
        
        cleanup {
            cleanWs()
        }
    }
}
