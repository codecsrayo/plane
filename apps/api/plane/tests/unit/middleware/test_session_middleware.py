# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

import pytest

from django.conf import settings
from django.http import HttpResponse
from django.test import RequestFactory

from plane.authentication.middleware.session import SessionMiddleware


@pytest.fixture
def middleware():
    return SessionMiddleware(lambda request: HttpResponse())


@pytest.fixture
def request_factory():
    return RequestFactory()


@pytest.mark.unit
class TestSessionMiddleware:
    def test_non_instance_paths_fallback_to_admin_cookie(
        self, middleware, request_factory
    ):
        request = request_factory.get("/api/users/me/")
        request.COOKIES[settings.ADMIN_SESSION_COOKIE_NAME] = "admin-session"

        middleware.process_request(request)

        assert request.session.session_key == "admin-session"

    def test_instance_paths_fallback_to_standard_cookie(
        self, middleware, request_factory
    ):
        request = request_factory.get("/api/instances/")
        request.COOKIES[settings.SESSION_COOKIE_NAME] = "app-session"

        middleware.process_request(request)

        assert request.session.session_key == "app-session"
