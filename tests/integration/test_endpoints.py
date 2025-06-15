import asyncio
import httpx
import pytest
from pydantic import BaseModel

ENDPOINT = "http://localhost:18989"


@pytest.mark.asyncio
async def test_echo_endpoint():
    client = httpx.AsyncClient(base_url=ENDPOINT)

    response = await client.get("/echo")
    assert response.status_code == 200, f"invalid response: {response}"
    assert response.text == ""

    json_data = {"hello": "world"}
    response = await client.post("/echo", json=json_data)
    assert response.status_code == 200, f"invalid response: {response}"
    assert response.json() == json_data


@pytest.mark.asyncio
async def test_json_endpoint():
    client = httpx.AsyncClient(base_url=ENDPOINT)
    content = {"hello": {"message": ["world"]}}

    response = await client.post("/json", json=content)
    assert response.status_code == 200, f"invalid response: {response}"
    assert response.headers["content-type"] == "application/json"
    assert response.json() == content


@pytest.mark.asyncio
async def test_interleaved_requests():
    class LongRunningResponse(BaseModel):
        request: dict
        handle_start: int
        handle_end: int

    async def do_request() -> LongRunningResponse:
        client = httpx.AsyncClient(base_url=ENDPOINT)
        response = await client.post("/long-running", json={"hello": "world"})
        assert response.status_code == 200, f"invalid response: {response}"
        return LongRunningResponse(**response.json())

    results = await asyncio.gather(
        *[do_request() for _ in range(10)],
    )

    assert (
        max(result.handle_start for result in results)
        - min(result.handle_start for result in results)
    ) < 1000

    for result in results:
        assert result.request == {"hello": "world"}
        assert result.handle_start > 0
        assert 2000 > result.handle_end - result.handle_start >= 1000


@pytest.mark.asyncio
async def test_path_endpoint():
    client = httpx.AsyncClient(base_url=ENDPOINT)
    response = await client.get("/path/123/content/456/789")
    assert response.status_code == 200, f"invalid response: {response}"
    assert response.headers["content-type"] == "application/json"
    assert response.json() == {
        "id": "123",
        "another_field": "456",
        "final_field": "789",
    }


@pytest.mark.asyncio
async def test_extend_endpoint():
    client = httpx.AsyncClient(base_url=ENDPOINT)
    response = await client.post("/extend", json={"hello": "world"})
    assert response.status_code == 201, f"invalid response: {response}"
    assert response.headers["content-type"] == "application/json"

    assert response.headers["x-header-1"] == "header-1-2"
    assert response.headers["x-header-2"] == "header-2"
    assert response.headers["x-header-3"] == "header-3"
    assert response.headers["x-header-4"] == "header-4-1"
    assert response.headers["x-header-5"] == "header-5"
    assert response.headers["x-header-6"] == "header-6"

    assert response.json() == {"hello": "world"}


@pytest.mark.asyncio
async def test_counter_middleware():
    client = httpx.AsyncClient(base_url=ENDPOINT)

    response = await client.post(
        "/counter_protected/echo",
        json={"hello": "world"},
        headers={"X-Add-Value": "30"},
    )
    assert response.status_code == 200, f"invalid response: {response}"
    assert response.json() == {"hello": "world"}

    response = await client.post(
        "/counter_protected/json",
        json={"hello": "world"},
        headers={"X-Add-Value": "30"},
    )
    assert response.status_code == 200, f"invalid response: {response}"
    assert response.headers["content-type"] == "application/json"
    assert response.json() == {"hello": "world"}

    response = await client.post(
        "/counter_protected/echo",
        json={"hello": "world"},
        headers={"X-Add-Value": "50"},
    )
    assert response.status_code == 429, f"invalid response: {response}"
    assert response.json() == {"error": "Counter limit exceeded"}


@pytest.mark.asyncio
async def test_middleware_chaining():
    client = httpx.AsyncClient(base_url=ENDPOINT)

    response = await client.post("/just_second/echo", json={"hello": "world"})
    assert response.status_code == 200, f"invalid response: {response}"
    assert response.json() == {"counter": 2}

    response = await client.post(
        "/just_second/echo",
        json={"hello": "world"},
        headers={"X-Must-Be-Unset": "1"},
    )
    assert response.status_code == 400, f"invalid response: {response}"
    assert response.json() == {"error": "Header must be unset"}

    response = await client.post("/combined/echo", json={"hello": "world"})
    assert response.status_code == 200, f"invalid response: {response}"
    assert response.json() == {"counter": 3}
    assert response.headers["x-was-set"] == "false"

    response = await client.post(
        "/combined/echo",
        json={"hello": "world"},
        headers={"X-Must-Be-Unset": "1"},
    )
    assert response.headers["x-was-set"] == "true"
    assert response.status_code == 200, f"invalid response: {response}"
    assert response.json() == {"counter": 3}

    response = await client.post("/flat_combined/echo", json={"hello": "world"})
    assert response.status_code == 200, f"invalid response: {response}"
    assert response.json() == {"counter": 3}


@pytest.mark.asyncio
async def test_methods_endpoint():
    client = httpx.AsyncClient(base_url=ENDPOINT)
    response = await client.get("/methods")
    assert response.status_code == 200, f"invalid response: {response}"
    assert response.json() == {"method": "GET", "endpoint": "get_endpoint"}

    response = await client.post("/methods")
    assert response.status_code == 200, f"invalid response: {response}"
    assert response.json() == {"method": "POST", "endpoint": "post_endpoint"}

    response = await client.request("VOROJBA", "/methods")
    assert response.status_code == 200, f"invalid response: {response}"
    assert response.json() == {
        "method": "VOROJBA",
        "endpoint": "extension_first_endpoint",
    }

    response = await client.request("ONE_HELL_LONG_VOROJBA_EXTENSION", "/methods")
    assert response.status_code == 200, f"invalid response: {response}"
    assert response.json() == {
        "method": "ONE_HELL_LONG_VOROJBA_EXTENSION",
        "endpoint": "extension_second_endpoint",
    }
