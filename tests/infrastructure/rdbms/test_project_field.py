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

from aim.domain.project.field import FieldData, FieldKind
from aim.infrastructure.rdbms.project_field import ProjectItemRepository
from aim.util import IdGenerator
from aim.util.session_handler import AsyncSessionHandler


# Pytest fixture for creating a project field repository
@pytest.fixture
def project_field_repository(session_handler: AsyncSessionHandler):
    return ProjectItemRepository(session_handler)


# Test case for CURD operations
@pytest.mark.asyncio
async def test_project_field_repository_curd(
    project_field_repository: ProjectItemRepository, id_generator: IdGenerator[int]
):
    # Save a project field
    field_id = id_generator.generate()
    project_id = id_generator.generate()
    field = FieldData(
        id=field_id,
        project_id=project_id,
        name="Test Field",
        kind=FieldKind.NUMBER,
        default_value=10,
    )
    await project_field_repository.save(field)

    # Find the saved project field
    retrieved_field = await project_field_repository.find(field_id)
    assert retrieved_field is not None
    assert retrieved_field.id == field_id
    assert retrieved_field.project_id == project_id
    assert retrieved_field.name == "Test Field"
    assert retrieved_field.kind == FieldKind.NUMBER
    assert retrieved_field.default_value == 10

    # Try to find a non-existent project field
    non_existent_field = await project_field_repository.find(id_generator.generate())
    assert non_existent_field is None

    # Edit the project field
    updated_field = FieldData(
        id=field_id,
        project_id=project_id,
        name="Updated Field",
        kind=FieldKind.NUMBER,
        default_value=20.5,
    )
    await project_field_repository.save(updated_field)

    # Retrieve the project field again to check if the changes were applied
    updated_retrieved_field = await project_field_repository.find(field_id)
    assert updated_retrieved_field is not None
    assert updated_retrieved_field.id == field_id
    assert updated_retrieved_field.project_id == project_id
    assert updated_retrieved_field.name == "Updated Field"
    assert updated_retrieved_field.kind == FieldKind.NUMBER
    assert updated_retrieved_field.default_value == 20.5
