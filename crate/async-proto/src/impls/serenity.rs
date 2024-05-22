use {
    std::num::NonZeroU64,
    serenity::model::id::*,
    async_proto_derive::impl_protocol_for,
};

impl_protocol_for! {
    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type AttachmentId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type ApplicationId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type ChannelId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type EmojiId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type GenericId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type GuildId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type IntegrationId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type MessageId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type RoleId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type ScheduledEventId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type StickerId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type StickerPackId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type StickerPackBannerId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type SkuId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type UserId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type WebhookId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type AuditLogEntryId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type InteractionId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type CommandId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type CommandPermissionId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type CommandVersionId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type TargetId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type StageInstanceId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type RuleId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type ForumTagId;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serenity")))))]
    #[async_proto(via = NonZeroU64, clone)]
    type EntitlementId;
}
