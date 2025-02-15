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

from aim.domain.project.project import ProjectData
from aim.infrastructure.rdbms.project import ProjectRepository
from aim.util import AsyncSessionHandler, IdGenerator


# Pytest fixture for creating a project repository
@pytest.fixture
def project_repository(session_handler: AsyncSessionHandler):
    return ProjectRepository(session_handler)


# Test case for CURD operations
@pytest.mark.asyncio
async def test_project_repository_curd(
    project_repository: ProjectRepository, id_generator: IdGenerator[int]
):
    # Save a project
    project_id = id_generator.generate()
    project = ProjectData(id=project_id, organization_id=1, name="Test Project")
    await project_repository.save(project)

    # Find the saved project
    retrieved_project = await project_repository.find(project_id)
    assert retrieved_project is not None
    assert retrieved_project.id == project_id
    assert retrieved_project.organization_id == 1
    assert retrieved_project.name == "Test Project"

    # Try to find a non-existent project
    non_existent_project = await project_repository.find(id_generator.generate())
    assert non_existent_project is None

    # Edit the project
    project = ProjectData(id=project_id, organization_id=1, name="Updated Project")
    await project_repository.save(project)

    # Retrieve the project again to check if the changes were applied
    updated_retrieved_project = await project_repository.find(project_id)
    assert updated_retrieved_project is not None
    assert updated_retrieved_project.name == "Updated Project"


# Test case for listing projects by organization
@pytest.mark.asyncio
async def test_project_repository_list_by_organization(
    project_repository: ProjectRepository, id_generator: IdGenerator[int]
):
    # First, save some projects for organization 3
    project_id1 = id_generator.generate()
    project1 = ProjectData(id=project_id1, organization_id=3, name="Project 1")
    await project_repository.save(project1)

    project_id2 = id_generator.generate()
    project2 = ProjectData(id=project_id2, organization_id=3, name="Project 2")
    await project_repository.save(project2)

    # Save a project for a different organization
    project_id3 = id_generator.generate()
    project3 = ProjectData(id=project_id3, organization_id=4, name="Project 3")
    await project_repository.save(project3)

    # Now, list projects for organization 3
    projects = await project_repository.list_by_organization(
        organization_id=3, offset=0, limit=10
    )
    assert len(projects) == 2
    assert projects[0].organization_id == 3
    assert projects[1].organization_id == 3
    assert {projects[0].name, projects[1].name} == {"Project 1", "Project 2"}

    # List projects with offset and limit
    projects = await project_repository.list_by_organization(
        organization_id=3, offset=1, limit=1
    )
