from django.urls import path


from plane.app.views import UnsplashEndpoint
from plane.app.views import GPTIntegrationEndpoint, RephraseGrammarEndpoint, WorkspaceGPTIntegrationEndpoint
from plane.app.views.external.sync import GitHubWebhookEndpoint, GitLabWebhookEndpoint


urlpatterns = [
    path("unsplash/", UnsplashEndpoint.as_view(), name="unsplash"),
    path("github-webhook/", GitHubWebhookEndpoint.as_view(), name="github-webhook"),
    path("gitlab-webhook/", GitLabWebhookEndpoint.as_view(), name="gitlab-webhook"),
    path(
        "workspaces/<str:slug>/projects/<uuid:project_id>/ai-assistant/",
        GPTIntegrationEndpoint.as_view(),
        name="importer",
    ),
    path(
        "workspaces/<str:slug>/ai-assistant/",
        WorkspaceGPTIntegrationEndpoint.as_view(),
        name="importer",
    ),
    path(
        "workspaces/<str:slug>/rephrase-grammar/",
        RephraseGrammarEndpoint.as_view(),
        name="rephrase-grammar",
    ),
]
