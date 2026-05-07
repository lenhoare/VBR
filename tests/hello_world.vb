Form "Hello World"

    Label "Enter your name and press OK."

    TextBox
        Label       := "Name:"
        Placeholder := "e.g. Alice"
        Binding     := userName
        OnChange    := NameChanged

    Row Align := Right
        Button "Cancel"
            OnClick := Cancel
            Style   := Danger

        Button "OK"
            OnClick := Ok
            Style   := Primary
    End Row

    StatusBar
        Binding := statusMessage

End Form


' --- Event handlers ---

Function NameChanged(value As String)
    If value Then
        statusMessage = "Hello, " & value & "!"
    Else
        statusMessage = "Type a name above."
    End If
End Function

Function Ok()
    statusMessage = "Welcome, " & userName & "!"
End Function

Function Cancel()
End Function
