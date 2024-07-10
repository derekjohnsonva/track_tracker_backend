# Test that we can post data to the event endpoint
import requests
import uuid as UUID


def test_competition():
    url = "http://localhost:3000/competitions"
    data = {
        "name": "test_name",
        "location": "test_location",
        "start_date": "2021-01-01",
        "end_date": "2021-01-02",
    }
    response = requests.post(url, json=data)
    assert response.status_code == 200
    uuid = response.json()["id"]

    url = "http://localhost:3000/competitions/" + uuid

    response = requests.get(url)
    assert response.status_code == 200
    assert response.json()["name"] == "test_name"

    response = requests.delete(url)
    assert response.status_code == 200

    response = requests.get(url)
    assert response.status_code == 404


def test_competition_bad_date():
    url = "http://localhost:3000/competitions"
    data = {
        "name": "test_name",
        "location": "test_location",
        "start_date": "2021-01-01",
        "end_date": "2020-01-022",  # this is a bad date
    }
    response = requests.post(url, json=data)
    assert response.status_code == 422


def test_competition_bad_data():
    url = "http://localhost:3000/competitions"
    data = {
        "name": "test_name",
        "location": "test_location",
        "start_date": "2021-01-01",
        "end_date": "2021-01-02",
        "bad_data": "bad_data",  # this is not a valid field
    }
    response = requests.post(url, json=data)
    print("response is " + str(response))
    assert response.status_code == 200


def test_event_bad_date():
    url = "http://localhost:3000/events"
    data = {
        "competition_id": str(UUID.uuid4()),
        "athlete_id": str(UUID.uuid4()),
        "name": "test_name",
        "date_time": "2022-12-19T16:39:57-08:",  # invalid date
    }
    response = requests.post(url, json=data)
    assert response.status_code == 422


def test_event():
    url = "http://localhost:3000/events"
    data = {
        "competition_id": str(UUID.uuid4()),
        "athlete_id": str(UUID.uuid4()),
        "name": "test_name",
        "date_time": "2022-12-19T16:39:57-08:00",
    }

    response = requests.post(url, json=data)
    assert response.status_code == 200
    uuid = response.json()["id"]

    url = "http://localhost:3000/events/" + uuid

    response = requests.get(url)
    assert response.status_code == 200
    assert response.json()["name"] == "test_name"

    response = requests.delete(url)
    assert response.status_code == 200

    response = requests.get(url)
    assert response.status_code == 404


# Test for User Endpoint
def test_user_endpoints():
    # Create a new user
    user_data = {
        "username": "John Doe",
        "athletes_following": [str(UUID.uuid4()), str(UUID.uuid4())],
    }
    response = requests.post("http://localhost:3000/users", json=user_data)
    assert response.status_code == 200
    user_id = response.json()["id"]

    # Retrieve the created user
    url = f"http://localhost:3000/users/{user_id}"
    response = requests.get(url)
    assert response.status_code == 200
    assert response.json()["username"] == "John Doe"

    # Delete the user
    response = requests.delete(url)
    assert response.status_code == 200

    # Verify the user is deleted
    response = requests.get(url)
    assert response.status_code == 404


# Test for Athlete Endpoint
def test_athlete_endpoints():
    # Create a new athlete
    athlete_data = {
        "first_name": "Jane",
        "last_name": "Doe",
        "bio": "Running Bio",
        "birthday": "1990-01-01",
    }
    response = requests.post("http://localhost:3000/athletes", json=athlete_data)
    assert response.status_code == 200
    athlete_id = response.json()["id"]

    # Retrieve the created athlete
    url = f"http://localhost:3000/athletes/{athlete_id}"
    response = requests.get(url)
    assert response.status_code == 200
    assert response.json()["first_name"] == "Jane"
    assert response.json()["last_name"] == "Doe"

    # Delete the athlete
    response = requests.delete(url)
    assert response.status_code == 200

    # Verify the athlete is deleted
    response = requests.get(url)
    assert response.status_code == 404
