export type ReactionType = {
  kind: "custom";
  animated: boolean;
  id: string;
  name: string | null;
} | { kind: "unicode"; name: string };
