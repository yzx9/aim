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


from aim.application.authorization import AccessTokenPayload, make_authorization
from aim.domain.user import Users

__all__ = ["Application"]


class Application:
    def __init__(self, *, users: Users) -> None:
        super().__init__()

        jwt_secret = ""
        session, auth_required = make_authorization(users, jwt_secret)

        self.session = session

        @auth_required
        def test(payload: AccessTokenPayload) -> str:
            return ""

        test("")
