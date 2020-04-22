from locust import Locust, TaskSet, task, between, HttpLocust

import random
import string
import json

def randomString(stringLength=10):
    """Generate a random string of fixed length """
    letters = string.ascii_lowercase
    return ''.join(random.choice(letters) for i in range(stringLength))

class SimpleSet(TaskSet):
    headers = {'content-type': 'application/json'}
    
    def on_start(self):
        user_name = randomString()
        request = json.dumps({"name": user_name})
        resp = self.client.post("/create_user", data = request, headers = self.headers)
        assert resp.status_code is 200
        json_resp = json.loads(resp.text)
        self.user_id = json_resp['id']
        
    @task(10)
    def create_user(self):
        resp = self.client.get("/user/%s" % self.user_id)
        assert resp.status_code is 200
    
class User(HttpLocust):
    task_set = SimpleSet
    wait_time = between(1, 4)
