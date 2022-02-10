import type { EmbedProvider } from "./EmbedProvider";
import type { EmbedVideo } from "./EmbedVideo";
import type { EmbedFooter } from "./EmbedFooter";
import type { EmbedThumbnail } from "./EmbedThumbnail";
import type { EmbedField } from "./EmbedField";
import type { EmbedAuthor } from "./EmbedAuthor";
import type { EmbedImage } from "./EmbedImage";

export interface Embed {
  author?: EmbedAuthor;
  color?: number;
  description?: string;
  fields?: Array<EmbedField>;
  footer?: EmbedFooter;
  image?: EmbedImage;
  kind?: string;
  provider?: EmbedProvider;
  thumbnail?: EmbedThumbnail;
  timestamp?: number;
  title?: string;
  url?: string;
  video?: EmbedVideo;
}
