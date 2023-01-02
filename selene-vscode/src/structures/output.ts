import { Diagnostic } from "./diagnostic"

export type Output =
    | ({
          type: "Diagnostic"
      } & Diagnostic)
    | {
          type: "PluginsNotLoaded"
          canon_filename: string
      }
