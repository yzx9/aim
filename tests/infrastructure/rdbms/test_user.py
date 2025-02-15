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

from aim.domain.user.user import UserData
from aim.infrastructure.rdbms.user import UserRepository
from aim.util import AsyncSessionHandler, IdGenerator


# Pytest fixture for creating a user repository
@pytest.fixture
def user_repository(session_handler: AsyncSessionHandler):
    return UserRepository(session_handler)


# Test case for CURD operations
@pytest.mark.asyncio
async def test_user_repository_curd(
    user_repository: UserRepository, id_generator: IdGenerator[int]
):
    # Save a user
    user_id = id_generator.generate()
    user = UserData(
        id=user_id,
        name="Test User",
        password_type="md5",
        password_hash="placeholder",
    )
    await user_repository.save(user)

    # Find the saved user
    retrieved_user = await user_repository.find(user_id)
    assert retrieved_user is not None
    assert retrieved_user.id == user_id
    assert retrieved_user.name == "Test User"

    # Try to find a non-existent user
    non_existent_user = await user_repository.find(id_generator.generate())
    assert non_existent_user is None

    # Edit the user
    updated_user = UserData(
        id=user_id,
        name="Updated User",
        password_type="sha256",
        password_hash="updated_hash",
    )
    await user_repository.save(updated_user)

    # Retrieve the user again to check if the changes were applied
    updated_retrieved_user = await user_repository.find(user_id)
    assert updated_retrieved_user is not None
    assert updated_retrieved_user.id == user_id
    assert updated_retrieved_user.name == "Updated User"
    assert updated_retrieved_user.password_type == "sha256"
    assert updated_retrieved_user.password_hash == "updated_hash"
