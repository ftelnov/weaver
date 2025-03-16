import pytest
import httpx

@pytest.mark.asyncio
async def test_mirror_endpoints(test_app):
    client = httpx.AsyncClient(base_url=test_app)

    response = await client.get("/mirror")
    assert response.status_code == 200, f"invalid response: {response.json()}"
    assert response.json() == {"detail": "Authentication header was not provided"}
