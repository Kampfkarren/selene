import { type Capabilities } from "./capabilities"
import { type Diagnostic } from "./diagnostic"

export type Output =
    | ({
          type: "Diagnostic"
      } & Diagnostic)
    | {
          type: "InvalidConfig"
          error: string
          source: string
          range?: {
              start: number
              end: number
          }
      }
    | ({
          type: "Capabilities"
      } & Capabilities)
