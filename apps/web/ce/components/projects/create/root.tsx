/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useState } from "react";
import { observer } from "mobx-react";
import { FormProvider, useForm } from "react-hook-form";
// plane imports
import { useTranslation } from "@plane/i18n";
import { TOAST_TYPE, setToast } from "@plane/propel/toast";
import { EFileAssetType } from "@plane/types";
// components
import ProjectCommonAttributes from "@/components/project/create/common-attributes";
import ProjectCreateHeader from "@/components/project/create/header";
import ProjectCreateButtons from "@/components/project/create/project-create-buttons";
// hooks
import { getCoverImageType, uploadCoverImage } from "@/helpers/cover-image.helper";
import { useProject } from "@/hooks/store/use-project";
import { usePlatformOS } from "@/hooks/use-platform-os";
// plane web types
import type { TProject } from "@/plane-web/types/projects";
import { ProjectAttributes } from "./attributes";
import { getProjectFormValues } from "./utils";

export type TCreateProjectFormProps = {
  setToFavorite?: boolean;
  workspaceSlug: string;
  onClose: () => void;
  handleNextStep: (projectId: string) => void;
  data?: Partial<TProject>;
  templateId?: string;
  updateCoverImageStatus: (projectId: string, coverImage: string) => Promise<void>;
};

export const CreateProjectForm = observer(function CreateProjectForm(props: TCreateProjectFormProps) {
  const { setToFavorite, workspaceSlug, data, onClose, handleNextStep, updateCoverImageStatus } = props;
  // store
  const { t } = useTranslation();
  const { addProjectToFavorites, createProject, updateProject } = useProject();
  // states
  const [shouldAutoSyncIdentifier, setShouldAutoSyncIdentifier] = useState(true);
  // form info
  const methods = useForm<TProject>({
    defaultValues: { ...getProjectFormValues(), ...data },
    reValidateMode: "onChange",
  });
  const { handleSubmit, reset, setValue } = methods;
  const { isMobile } = usePlatformOS();
  const handleAddToFavorites = (projectId: string) => {
    if (!workspaceSlug) return;

    addProjectToFavorites(workspaceSlug.toString(), projectId).catch(() => {
      setToast({
        type: TOAST_TYPE.ERROR,
        title: t("toast.error"),
        message: t("failed_to_remove_project_from_favorites"),
      });
    });
  };

  const onSubmit = async (formData: Partial<TProject>) => {
    const payload: Partial<TProject> = { ...formData };

    // Upper case identifier
    payload.identifier = payload.identifier?.toUpperCase();
    const coverImage = payload.cover_image_url;
    const imageType = coverImage ? getCoverImageType(coverImage) : null;
    const needsCoverUpload = imageType === "local_static" || imageType === "unsplash";
    let uploadedAssetUrl: string | null = null;

    delete payload.cover_image_url;

    if (coverImage && !needsCoverUpload) {
      payload.cover_image = coverImage;
      payload.cover_image_asset = null;
    }

    return createProject(workspaceSlug.toString(), payload)
      .then(async (res) => {
        let coverUploadFailed = false;
        if (coverImage && needsCoverUpload) {
          try {
            uploadedAssetUrl = await uploadCoverImage(coverImage, {
              workspaceSlug: workspaceSlug.toString(),
              entityIdentifier: res.id,
              entityType: EFileAssetType.PROJECT_COVER,
              isUserAsset: false,
            });
          } catch (error) {
            // El proyecto ya se creo; no abortamos el flujo por un
            // fallo de asset secundario. Avisamos al usuario y seguimos
            // con la navegacion/redireccion normales.
            coverUploadFailed = true;
            console.error("Error uploading cover image:", error);
            setToast({
              type: TOAST_TYPE.WARNING,
              title: t("toast.warning"),
              message: error instanceof Error ? error.message : "Failed to upload cover image",
            });
          }
        }

        try {
          if (uploadedAssetUrl) {
            await updateCoverImageStatus(res.id, uploadedAssetUrl);
            await updateProject(workspaceSlug.toString(), res.id, { cover_image_url: uploadedAssetUrl });
          } else if (coverImage && !needsCoverUpload) {
            await updateCoverImageStatus(res.id, coverImage);
            await updateProject(workspaceSlug.toString(), res.id, { cover_image_url: coverImage });
          }
        } catch (error) {
          // Tampoco abortamos por fallos post-creacion del cover.
          console.error("Error updating cover image metadata:", error);
        }

        if (!coverUploadFailed) {
          setToast({
            type: TOAST_TYPE.SUCCESS,
            title: t("success"),
            message: t("project_created_successfully"),
          });
        }

        if (setToFavorite) {
          handleAddToFavorites(res.id);
        }
        handleNextStep(res.id);
        return res;
      })
      .catch((err) => {
        try {
          // Handle the new error format where codes are nested in arrays under field names
          const errorData = err?.data ?? {};

          const nameError = errorData.name?.includes("PROJECT_NAME_ALREADY_EXIST");
          const identifierError = errorData?.identifier?.includes("PROJECT_IDENTIFIER_ALREADY_EXIST");

          if (nameError || identifierError) {
            if (nameError) {
              setToast({
                type: TOAST_TYPE.ERROR,
                title: t("toast.error"),
                message: t("project_name_already_taken"),
              });
            }

            if (identifierError) {
              setToast({
                type: TOAST_TYPE.ERROR,
                title: t("toast.error"),
                message: t("project_identifier_already_taken"),
              });
            }
          } else {
            setToast({
              type: TOAST_TYPE.ERROR,
              title: t("toast.error"),
              message: t("something_went_wrong"),
            });
          }
        } catch (error) {
          // Fallback error handling if the error processing fails
          console.error("Error processing API error:", error);
          setToast({
            type: TOAST_TYPE.ERROR,
            title: t("toast.error"),
            message: t("something_went_wrong"),
          });
        }
      });
  };

  const handleClose = () => {
    onClose();
    setShouldAutoSyncIdentifier(true);
    setTimeout(() => {
      reset();
    }, 300);
  };

  return (
    <FormProvider {...methods}>
      <ProjectCreateHeader handleClose={handleClose} isMobile={isMobile} />

      <form onSubmit={handleSubmit(onSubmit)} className="px-3">
        <div className="mt-9 space-y-6 pb-5">
          <ProjectCommonAttributes
            setValue={setValue}
            isMobile={isMobile}
            shouldAutoSyncIdentifier={shouldAutoSyncIdentifier}
            setShouldAutoSyncIdentifier={setShouldAutoSyncIdentifier}
          />
          <ProjectAttributes isMobile={isMobile} />
        </div>
        <ProjectCreateButtons handleClose={handleClose} />
      </form>
    </FormProvider>
  );
});
