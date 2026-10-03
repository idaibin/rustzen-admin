import { useRef, useState } from "react";

import { t } from "@/lib/i18n";

/** A synchronous write lock; a loading button alone cannot guard form submission. */
export function useSubmission() {
    const pending = useRef(false);
    const [submitting, setSubmitting] = useState(false);
    const [submissionError, setSubmissionError] = useState<string>();

    const beginSubmission = () => {
        if (pending.current) return false;
        pending.current = true;
        setSubmissionError(undefined);
        setSubmitting(true);
        return true;
    };

    const finishSubmission = () => {
        pending.current = false;
        setSubmitting(false);
    };

    const failSubmission = (error: unknown) => {
        setSubmissionError(
            error instanceof Error && error.message
                ? error.message
                : t("操作失败，请检查后重试。", "The action failed. Please check and retry."),
        );
    };

    const clearSubmissionError = () => setSubmissionError(undefined);

    return {
        submitting,
        beginSubmission,
        finishSubmission,
        submissionError,
        failSubmission,
        clearSubmissionError,
    };
}
