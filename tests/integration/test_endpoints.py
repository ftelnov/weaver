import asyncio
import httpx
import pytest
from pydantic import BaseModel


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


@pytest.mark.asyncio
async def test_json_endpoint(test_app):
    client = httpx.AsyncClient(base_url=test_app)
    content = {"hello": {"message": ["world"]}}

    response = await client.post("/json", json=content)
    assert response.status_code == 200, f"invalid response: {response}"
    assert response.headers["content-type"] == "application/json"
    assert response.json() == content


@pytest.mark.asyncio
async def test_interleaved_requests(test_app):
    class LongRunningResponse(BaseModel):
        request: dict
        handle_start: int
        handle_end: int

    async def do_request() -> LongRunningResponse:
        client = httpx.AsyncClient(base_url=test_app)
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
