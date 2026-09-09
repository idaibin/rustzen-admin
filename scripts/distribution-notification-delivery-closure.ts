export function assertNotificationDeliveryClosure({
    hasNotifications,
    retainedText,
    generatedText,
    outputText,
}: {
    hasNotifications: boolean;
    retainedText: string;
    generatedText: string;
    outputText: string;
}) {
    const endpoint = "/api/monitor/notification-delivery";
    const cardTestId = "notification-delivery-card";
    const retainedNotify =
        retainedText.includes("monitorAPI") &&
        retainedText.includes("notification marker");
    const generatedCard = generatedText.includes("NotificationDeliveryCard");
    const distEndpoint = outputText.includes(endpoint);
    const distCard = outputText.includes(cardTestId);
    const complete =
        retainedNotify && generatedCard && distEndpoint && distCard;
    const absent =
        retainedText.includes("monitorCoreAPI") &&
        !retainedText.includes("notification marker") &&
        !generatedCard &&
        !distEndpoint &&
        !distCard;
    if (hasNotifications ? !complete : !absent)
        throw new Error(
            "selected notification delivery closure differs from preset",
        );
}

export function assertSelectedApiAuthority({
    authoritative,
    generated,
    retained,
}: {
    authoritative: Uint8Array;
    generated: Uint8Array;
    retained: Uint8Array;
}) {
    if (
        !sameBytes(authoritative, generated) ||
        !sameBytes(authoritative, retained)
    )
        throw new Error(
            "selected Web API sources differ from the authoritative template",
        );
}

function sameBytes(left: Uint8Array, right: Uint8Array) {
    return (
        left.length === right.length &&
        left.every((byte, index) => byte === right[index])
    );
}
