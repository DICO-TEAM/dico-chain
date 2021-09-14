pipeline {
    agent any
    stages {
        stage('build'){
            steps {
                echo "-----------------make install---------------------------"
                sh 'make build-dev'
                echo "-----------------make install---------------------------"
            }
        }
        stage('kill'){
            steps {
                echo "-----------------make kill---------------------------"
                sh 'make kill'
                echo "-----------------make kill---------------------------"
            }
        }
        stage('deploy'){
            steps {
                echo "-----------------deploy start ---------------------------"
                sh 'JENKINS_NODE_COOKIE=dontKillMe nohup make dev >> chain.log 2>&1 &'
                echo "-----------------deploy end---------------------------"
            }
        }
    }

    post {
        always {
            emailext (
				subject: '$DEFAULT_SUBJECT',
                body: '$DEFAULT_CONTENT',
                recipientProviders: [
                    [$class: 'CulpritsRecipientProvider'],
                    [$class: 'DevelopersRecipientProvider'],
                    [$class: 'RequesterRecipientProvider']
                ],
                to: '$DEFAULT_RECIPIENTS'
            )
        }
    }
}
