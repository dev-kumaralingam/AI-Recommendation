from flask import Flask, jsonify
from flask_graphql import GraphQLView
import graphene
import requests
import os
from dotenv import load_dotenv

load_dotenv()

app = Flask(__name__)

# Environment variable for Groq API Key
GROQ_API_KEY = os.getenv('GROQ_API_KEY')
GROQ_API_URL = "https://api.groq.com/openai/v1/chat/completions"

# Define the Query class for GraphQL
class Query(graphene.ObjectType):
    query_ai = graphene.String(query=graphene.String(required=True))

    def resolve_query_ai(self, info, query):
        return query_groq(query)

def query_groq(user_query):
    headers = {
        "Authorization": f"Bearer {GROQ_API_KEY}",
        "Content-Type": "application/json"
    }

    payload = {
        "model": "mixtral-8x7b-32768",
        "messages": [{"role": "user", "content": user_query}],
        "temperature": 0.7
    }

    try:
        response = requests.post(GROQ_API_URL, headers=headers, json=payload)
        response.raise_for_status()
        groq_response = response.json()
        
        return groq_response['choices'][0]['message']['content']

    except requests.exceptions.RequestException as e:
        return f"Error communicating with Groq API: {str(e)}"

# Set up the GraphQL schema
schema = graphene.Schema(query=Query)

# Add the GraphQL endpoint
app.add_url_rule('/graphql', view_func=GraphQLView.as_view('graphql', schema=schema, graphiql=True))

@app.route('/')
def index():
    return jsonify({"message": "Welcome to the AI Recommendation GraphQL API"})

if __name__ == '__main__':
    app.run(host='127.0.0.1', port=8080, debug=True)
