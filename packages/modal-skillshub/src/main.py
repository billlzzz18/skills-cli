import modal
import logging
from pathlib import Path
from typing import Any, Optional

from fastapi import FastAPI, Depends, HTTPException, status, Request
from fastapi.security import APIKeyHeader

from skills_hub_client import install_skill_from_url, HubInstallResult
from skills_manager import SkillService, SkillInfo, MODAL_SKILLS_ROOT, BUILTIN_SKILLS_DIR, CUSTOMIZED_SKILLS_DIR, ACTIVE_SKILLS_DIR

logger = logging.getLogger(__name__)

# Define a Modal Volume for persistent storage of skills
skills_volume = modal.Volume.from_name("manus-skills-volume", create_if_missing=True)

# Define a Modal Secret for API keys
# You need to create this secret in Modal dashboard or via CLI:
# modal secret create my-api-keys API_KEY="your_secret_api_key_here"
api_keys_secret = modal.Secret.from_name("manus-api-keys")

# Define a Modal Image with necessary dependencies
image = (
    modal.Image.debian_slim(python_version="3.11")
    .uv_pip_install("requests", "frontmatter", "pydantic", "fastapi", "uvicorn")
)

# Define the Modal App
app = modal.App("skills-hub-client", image=image, volumes={MODAL_SKILLS_ROOT: skills_volume}, secrets=[api_keys_secret])

# FastAPI app for web endpoints
web_app = FastAPI()

# API Key authentication dependency
api_key_header = APIKeyHeader(name="X-API-Key", auto_error=False)

async def get_api_key(api_key: str = Depends(api_key_header)):
    if api_key is None:
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="X-API-Key header missing")
    
    valid_api_key = os.environ.get("API_KEY")
    if not valid_api_key:
        logger.error("API_KEY environment variable not set in Modal Secret.")
        raise HTTPException(status_code=status.HTTP_500_INTERNAL_SERVER_ERROR, detail="Server API key not configured.")

    if api_key != valid_api_key:
        raise HTTPException(status_code=status.HTTP_403_FORBIDDEN, detail="Invalid API Key")
    return api_key


@app.function()
def modal_install_skill_from_url(
    bundle_url: str,
    version: str = "",
    overwrite: bool = False,
    enable: bool = True,
    webhook_url: Optional[str] = None,
) -> HubInstallResult:
    logger.info(f"Modal function called to install skill from {bundle_url}")
    result = install_skill_from_url(bundle_url, version, overwrite, enable)

    if webhook_url:
        # Send result to webhook asynchronously
        modal_send_webhook_result.remote(webhook_url, result.dict())

    return result

@app.function()
def modal_send_webhook_result(webhook_url: str, result_data: dict):
    logger.info(f"Sending webhook result to {webhook_url}")
    try:
        import requests
        response = requests.post(webhook_url, json=result_data, timeout=10)
        response.raise_for_status()
        logger.info(f"Webhook sent successfully to {webhook_url}")
    except requests.exceptions.RequestException as e:
        logger.error(f"Failed to send webhook to {webhook_url}: {e}")


@web_app.post("/api/scan/lookup")
async def scan_lookup_endpoint(request: Request, api_key: str = Depends(get_api_key)):
    """
    Endpoint for scanning and looking up a skill from a URL.
    Supports payload format: {"skillUrl": "..."}
    """
    data = await request.json()
    skill_url = data.get("skillUrl") or data.get("skill_url")
    
    if not skill_url:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="'skillUrl' is required")

    # Optional parameters that can still be passed
    version = data.get("version", "")
    overwrite = data.get("overwrite", False)
    enable = data.get("enable", True)
    webhook_url = data.get("webhook_url")

    # Call the Modal function asynchronously to install/lookup the skill
    modal_install_skill_from_url.spawn(skill_url, version, overwrite, enable, webhook_url)
    
    return {
        "status": "success",
        "message": "Skill scan/lookup initiated",
        "skillUrl": skill_url
    }


@web_app.post("/install-skill")
async def install_skill_endpoint(request: Request, api_key: str = Depends(get_api_key)):
    data = await request.json()
    bundle_url = data.get("bundle_url") or data.get("skillUrl")
    version = data.get("version", "")
    overwrite = data.get("overwrite", False)
    enable = data.get("enable", True)
    webhook_url = data.get("webhook_url")

    if not bundle_url:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="'bundle_url' or 'skillUrl' is required")

    # Call the Modal function asynchronously
    modal_install_skill_from_url.spawn(bundle_url, version, overwrite, enable, webhook_url)
    
    return {"message": "Skill installation initiated", "bundle_url": bundle_url}


@app.function()
def modal_list_available_skills() -> list[SkillInfo]:
    logger.info("Modal function called to list available skills.")
    return SkillService.list_available_skills()

@web_app.get("/list-skills")
async def list_skills_endpoint(api_key: str = Depends(get_api_key)):
    skills = modal_list_available_skills.remote()
    return [skill.dict() for skill in skills]


@app.function()
def modal_get_skill_info(name: str) -> SkillInfo | None:
    logger.info(f"Modal function called to get info for skill: {name}")
    return SkillService.get_skill_info(name)

@web_app.get("/skill-info/{skill_name}")
async def get_skill_info_endpoint(skill_name: str, api_key: str = Depends(get_api_key)):
    skill_info = modal_get_skill_info.remote(skill_name)
    if skill_info is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Skill not found")
    return skill_info.dict()


@app.function()
def modal_enable_skill(name: str, force: bool = False) -> bool:
    logger.info(f"Modal function called to enable skill: {name}")
    return SkillService.enable_skill(name, force)

@web_app.post("/enable-skill")
async def enable_skill_endpoint(request: Request, api_key: str = Depends(get_api_key)):
    data = await request.json()
    name = data.get("name")
    force = data.get("force", False)
    if not name:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="'name' is required")
    success = modal_enable_skill.remote(name, force)
    if not success:
        raise HTTPException(status_code=status.HTTP_500_INTERNAL_SERVER_ERROR, detail=f"Failed to enable skill {name}")
    return {"message": f"Skill {name} enabled successfully"}


@app.function()
def modal_disable_skill(name: str) -> bool:
    logger.info(f"Modal function called to disable skill: {name}")
    return SkillService.disable_skill(name)

@web_app.post("/disable-skill")
async def disable_skill_endpoint(request: Request, api_key: str = Depends(get_api_key)):
    data = await request.json()
    name = data.get("name")
    if not name:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="'name' is required")
    success = modal_disable_skill.remote(name)
    if not success:
        raise HTTPException(status_code=status.HTTP_500_INTERNAL_SERVER_ERROR, detail=f"Failed to disable skill {name}")
    return {"message": f"Skill {name} disabled successfully"}


@app.function()
def modal_delete_skill(name: str) -> bool:
    logger.info(f"Modal function called to delete skill: {name}")
    return SkillService.delete_skill(name)

@web_app.delete("/delete-skill/{skill_name}")
async def delete_skill_endpoint(skill_name: str, api_key: str = Depends(get_api_key)):
    success = modal_delete_skill.remote(skill_name)
    if not success:
        raise HTTPException(status_code=status.HTTP_500_INTERNAL_SERVER_ERROR, detail=f"Failed to delete skill {skill_name}")
    return {"message": f"Skill {skill_name} deleted successfully"}


@app.function()
def modal_sync_from_active_to_customized(skill_names: list[str] | None = None) -> tuple[int, int]:
    logger.info("Modal function called to sync active skills to customized.")
    return SkillService.sync_from_active_to_customized(skill_names)

@web_app.post("/sync-active-to-customized")
async def sync_active_to_customized_endpoint(request: Request, api_key: str = Depends(get_api_key)):
    data = await request.json()
    skill_names = data.get("skill_names")
    synced_count, skipped_count = modal_sync_from_active_to_customized.remote(skill_names)
    return {"message": f"Synced {synced_count} skills, skipped {skipped_count} skills"}


@app.function()
def modal_load_skill_file(skill_name: str, file_path: str, source: str) -> str | None:
    logger.info(f"Modal function called to load file {file_path} from skill {skill_name} ({source}).")
    return SkillService.load_skill_file(skill_name, file_path, source)

@web_app.post("/load-skill-file")
async def load_skill_file_endpoint(request: Request, api_key: str = Depends(get_api_key)):
    data = await request.json()
    skill_name = data.get("skill_name")
    file_path = data.get("file_path")
    source = data.get("source")

    if not all([skill_name, file_path, source]):
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="'skill_name', 'file_path', and 'source' are required")
    
    content = modal_load_skill_file.remote(skill_name, file_path, source)
    if content is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="File not found or could not be loaded")
    return {"content": content}


@app.local_entrypoint()
def main():
    print("This is a local entrypoint for testing the Modal app.")
    print("To deploy this Modal app, run: modal deploy main.py")
    print("To run a remote function, use: modal run main.py::modal_install_skill_from_url --bundle-url <url>")
    print("You can also call other functions like modal_list_available_skills, modal_get_skill_info, etc.")
    print("Web endpoints are available after deployment at your-modal-user-name--skills-hub-client.modal.run/install-skill etc.")
    print("Remember to set up the 'manus-api-keys' secret with API_KEY environment variable.")

@app.web()
def fastapi_app():
    return web_app
