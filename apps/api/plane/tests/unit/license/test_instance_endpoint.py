import pytest
from django.urls import reverse
from django.utils import timezone
from rest_framework import status

from plane.license.models import Instance, InstanceConfiguration


@pytest.mark.django_db
class TestInstanceEndpoint:
    def test_instance_endpoint_returns_slack_enabled_flag_from_configuration(self, api_client):
        Instance.objects.create(
            instance_name="Test Instance",
            instance_id="test-instance-id",
            current_version="1.0.0",
            latest_version="1.0.0",
            last_checked_at=timezone.now(),
        )
        InstanceConfiguration.objects.create(
            key="IS_SLACK_ENABLED",
            value="0",
            category="SLACK",
            is_encrypted=False,
        )

        response = api_client.get(reverse("instance"))

        assert response.status_code == status.HTTP_200_OK
        assert response.json()["config"]["is_slack_enabled"] is False
