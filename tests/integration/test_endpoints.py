import httpx
import pytest


@pytest.mark.asyncio
async def test_echo_endpoint(test_app):
    client = httpx.AsyncClient(base_url=test_app)

    response = await client.get("/echo")
    assert response.status_code == 200, f"invalid response: {response}"
    assert response.text == ""

    json_data = {"hello": "world"}
    response = await client.post("/echo", json=json_data)
    assert response.status_code == 200, f"invalid response: {response}"
    assert response.json() == json_data
