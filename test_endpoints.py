# Test that we can post data to the event endpoint

import json
import pytest
import requests


def setup_competition_data():
    url = 'http://localhost:3000/competitions'
    response = requests.get(url)
    for competition in response.json():
        url = 'http://localhost:3000/competitions/' + competition["id"]
        response = requests.delete(url)


def test_competition():
    setup_competition_data()
    url = 'http://localhost:3000/competitions'
    data = {
        "name": "test_name",
        "location": "test_location",
        "start_date": "2021-01-01",
        "end_date": "2021-01-02"
    }
    response = requests.post(url, json=data)
    assert response.status_code == 200

    url = 'http://localhost:3000/competitions'
    response = requests.get(url)
    assert response.status_code == 200
    assert response.json()[0]["name"] == "test_name"
    uuid = response.json()[0]["id"]

    url = 'http://localhost:3000/competitions/' + uuid

    response = requests.get(url)
    assert response.status_code == 200
    assert response.json()["name"] == "test_name"

    response = requests.delete(url)
    assert response.status_code == 200

    response = requests.get(url)
    assert response.status_code == 404


def test_competition_bad_date():
    setup_competition_data()
    url = 'http://localhost:3000/competitions'
    data = {
        "name": "test_name",
        "location": "test_location",
        "start_date": "2021-01-01",
        "end_date": "2020-01-022"
    }
    response = requests.post(url, json=data)
    assert response.status_code == 422


def test_competition_bad_data():
    setup_competition_data()
    url = 'http://localhost:3000/competitions'
    data = {
        "name": "test_name",
        "location": "test_location",
        "start_date": "2021-01-01",
        "end_date": "2021-01-02",
        "bad_data": "bad_data"
    }
    response = requests.post(url, json=data)
    assert response.status_code == 200
