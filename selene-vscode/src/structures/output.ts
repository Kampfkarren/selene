import { Diagnostic } from "./diagnostic"

export type Output = {
    type: "diagnostic"
} & Diagnostic
