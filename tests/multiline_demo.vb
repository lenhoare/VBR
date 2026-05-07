Form "Note Editor"

    Label "Write your note below, then press Save."

    TextBox
        Label      := "Note:"
        Binding    := noteText
        MultiLine  := True
        ViewHeight := 8

    Row Align := Right
        Button "Clear"
            OnClick := Clear
            Style   := Normal

        Button "Cancel"
            OnClick := Cancel
            Style   := Danger

        Button "Save"
            OnClick := Save
            Style   := Primary
    End Row

    StatusBar
        Binding := statusMessage

End Form


' --- Event handlers ---

Function Save()
    If noteText Then
        statusMessage = "Saved " & noteText
    Else
        statusMessage = "Nothing to save."
    End If
End Function

Function Clear()
    noteText = ""
    statusMessage = "Note cleared."
End Function

Function Cancel()
End Function
