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
from aim.util import AsyncSessionHandler


# Pytest fixture for creating a user repository
@pytest.fixture
def user_repository(session_handler: AsyncSessionHandler):
    return UserRepository(session_handler)


# Test case for saving a user
@pytest.mark.asyncio
async def test_user_repository_save(user_repository: UserRepository):
    user = UserData(id=1, name="Test User")
    await user_repository.save(user)

    retrieved_user = await user_repository.find(1)
    assert retrieved_user is not None
    assert retrieved_user.id == 1
    assert retrieved_user.name == "Test User"


# Test case for finding a user
@pytest.mark.asyncio
async def test_user_repository_find(user_repository: UserRepository):
    # First, save a user
    user = UserData(id=2, name="Another User")
    await user_repository.save(user)

    # Now, try to find it
    retrieved_user = await user_repository.find(2)
    assert retrieved_user is not None
    assert retrieved_user.id == 2
    assert retrieved_user.name == "Another User"

    # Try to find a non-existent user
    non_existent_user = await user_repository.find(999)
    assert non_existent_user is None
