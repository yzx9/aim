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


import inspect
from typing import Callable, Concatenate, ParamSpec, TypeVar, overload

from aim.application.exceptions import (
    InvalidAuthorizationToken,
    MissingAuthorizationToken,
)
from aim.domain.user import AccessTokenPayload, Session, Users

__all__ = ["AuthorizationMiddleware"]


_P = ParamSpec("_P")
_R = TypeVar("_R")


class AuthorizationMiddleware:
    def __init__(self, users: Users):
        super().__init__()

        @overload
        def required(
            handler: Callable[Concatenate[Session, _P], _R],
        ) -> Callable[Concatenate[str | None, _P], _R]: ...

        @overload
        def required(
            handler: Callable[Concatenate[AccessTokenPayload, _P], _R],
        ) -> Callable[Concatenate[str | None, _P], _R]: ...

        @overload
        def required(
            handler: Callable[_P, _R],
        ) -> Callable[Concatenate[str | None, _P], _R]: ...

        def required(handler: Callable) -> Callable:
            """Authentication middleware as a function wrapper."""
            need_param = None
            sig = inspect.signature(handler)
            params = list(sig.parameters.values())

            # Skip 'self' parameter if it exists
            start_index = 0
            if params and str(params[0]).startswith("self"):
                start_index = 1

            if len(params) > start_index:
                first_param_name = list(sig.parameters.keys())[start_index]
                first_param_type = sig.parameters[first_param_name].annotation
                if first_param_type is Session:
                    need_param = "Session"
                elif first_param_type is AccessTokenPayload:
                    need_param = "AccessTokenPayload"

            def decorator(token: str | None, *args, **kwargs):
                """Wrapper function to extract and validate the token."""
                # Assuming the first argument is the token string
                if token is None or not isinstance(token, str):
                    raise MissingAuthorizationToken()

                session = users.recovery_session(token)
                if session is None:
                    raise InvalidAuthorizationToken()

                # Call the actual handler with the parsed payload
                match need_param:
                    case "Session":
                        return handler(session, *args, **kwargs)  # type: ignore
                    case "AccessTokenPayload":
                        return handler(session.access_payload, *args, **kwargs)  # type: ignore
                    case _:
                        return handler(*args, **kwargs)  # type: ignore

            return decorator

        self.required = required
