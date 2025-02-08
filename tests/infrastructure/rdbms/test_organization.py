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
from aim.util import AsyncSessionHandler


# Pytest fixture for creating an organization repository
@pytest.fixture
def organization_repository(session_handler: AsyncSessionHandler):
    return OrganizationRepository(session_handler)


# Test case for saving an organization
@pytest.mark.asyncio
async def test_organization_repository_save(
    organization_repository: OrganizationRepository,
):
    organization = OrganizationData(id=1, name="Test Organization")
    await organization_repository.save(organization)

    retrieved_organization = await organization_repository.find(1)
    assert retrieved_organization is not None
    assert retrieved_organization.id == 1
    assert retrieved_organization.name == "Test Organization"


# Test case for finding an organization
@pytest.mark.asyncio
async def test_organization_repository_find(
    organization_repository: OrganizationRepository,
):
    # First, save an organization
    organization = OrganizationData(id=2, name="Another Organization")
    await organization_repository.save(organization)

    # Now, try to find it
    retrieved_organization = await organization_repository.find(2)
    assert retrieved_organization is not None
    assert retrieved_organization.id == 2
    assert retrieved_organization.name == "Another Organization"

    # Try to find a non-existent organization
    non_existent_organization = await organization_repository.find(999)
    assert non_existent_organization is None
