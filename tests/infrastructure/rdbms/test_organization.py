# Copyright 2025 Zexin Yuan
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#    http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.


import pytest

from aim.domain.organization.organization import OrganizationData
from aim.infrastructure.rdbms.organization import OrganizationRepository
from aim.util import AsyncSessionHandler, IdGenerator


# Pytest fixture for creating an organization repository
@pytest.fixture
def organization_repository(session_handler: AsyncSessionHandler):
    return OrganizationRepository(session_handler)


# Test case for CURD operations
@pytest.mark.asyncio
async def test_organization_repository_curd(
    organization_repository: OrganizationRepository, id_generator: IdGenerator[int]
):
    # Save an organization
    organization_id = id_generator.generate()
    organization = OrganizationData(id=organization_id, name="Test Organization")
    await organization_repository.save(organization)

    # Find the saved organization
    retrieved_organization = await organization_repository.find(organization_id)
    assert retrieved_organization is not None
    assert retrieved_organization.id == organization_id
    assert retrieved_organization.name == "Test Organization"

    # Try to find a non-existent organization
    non_existent_organization = await organization_repository.find(
        id_generator.generate()
    )
    assert non_existent_organization is None

    # Edit the organization
    updated_organization = OrganizationData(
        id=organization_id, name="Updated Organization"
    )
    await organization_repository.save(updated_organization)

    # Retrieve the organization again to check if the changes were applied
    updated_retrieved_organization = await organization_repository.find(organization_id)
    assert updated_retrieved_organization is not None
    assert updated_retrieved_organization.id == organization_id
    assert updated_retrieved_organization.name == "Updated Organization"

    # Delete the project field
    await organization_repository.delete(organization_id)
    deleted_organization = await organization_repository.find(organization_id)
    assert deleted_organization is None
