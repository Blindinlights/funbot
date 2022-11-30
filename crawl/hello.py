import flask
import github
from flask import Flask , request , jsonify


app=Flask(__name__)

@app.route('/github_repo', methods=['POST'])
def get_github():
    data=request.get_json()
    repo_url=data['repo_url']
    repo_property=github.get_github_repo_property(repo_url)
    return jsonify(repo_property)

