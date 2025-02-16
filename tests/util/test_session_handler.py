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
import pytest_asyncio
import sqlalchemy as sa
from sqlalchemy.ext.asyncio import AsyncSession, create_async_engine
from sqlalchemy.orm import Mapped, declarative_base, mapped_column

from aim.util.session_handler import SQLAlchemyAsyncSessionHandler

Base = declarative_base()


class TestModel(Base):
    __test__ = False
    __tablename__ = "test_model"

    id: Mapped[int] = mapped_column(primary_key=True)
    name: Mapped[str]


@pytest_asyncio.fixture(scope="package")
async def test_engine():
    engine = create_async_engine("sqlite+aiosqlite:///:memory:")
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)
    yield engine
    await engine.dispose()


@pytest.mark.asyncio
async def test_session_handler_new_session_created(test_engine):
    session_handler = SQLAlchemyAsyncSessionHandler(test_engine)
    async with session_handler.session_handler() as session:
        assert isinstance(session, AsyncSession)


@pytest.mark.asyncio
async def test_session_handler_commit_on_success(test_engine):
    name = "commit_on_success"
    session_handler = SQLAlchemyAsyncSessionHandler(test_engine)
    async with session_handler.session_handler() as session:
        model = TestModel(name=name)
        session.add(model)

    async with AsyncSession(test_engine) as session:
        result = await session.execute(
            sa.select(TestModel).where(TestModel.name == name)
        )
        retrieved_model = result.scalars().first()
        assert retrieved_model is not None


@pytest.mark.asyncio
async def test_session_handler_rollback_on_exception(test_engine):
    name = "rollback_on_exception"
    session_handler = SQLAlchemyAsyncSessionHandler(test_engine)
    with pytest.raises(Exception):
        async with session_handler.session_handler() as session:
            model = TestModel(name=name)
            session.add(model)
            raise Exception("Simulated error")

    async with AsyncSession(test_engine) as session:
        result = await session.execute(
            sa.select(TestModel).where(TestModel.name == name)
        )
        retrieved_model = result.scalars().first()
        assert retrieved_model is None


@pytest.mark.asyncio
async def test_session_handler_uses_provided_session(test_engine):
    name = "uses_provided_session"
    session_handler = SQLAlchemyAsyncSessionHandler(test_engine)
    async with AsyncSession(test_engine) as existing_session:
        async with session_handler.session_handler(existing_session) as session:
            assert session is existing_session
            model = TestModel(name=name)
            session.add(model)

        # Existing session should still be open
        assert existing_session.is_active

        await existing_session.commit()
