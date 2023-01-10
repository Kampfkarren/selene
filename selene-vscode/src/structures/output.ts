import { Diagnostic } from "./diagnostic"

export type Output =
    | ({
          type: "Diagnostic"
      } & Diagnostic)
    | {
          type: "InvalidConfig"
          error: string
          source: string
          location?: { line: number }
      }
