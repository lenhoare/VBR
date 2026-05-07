Form "Connection Setup"

    Label "VBR Connection Setup"
        Style := Bold

    Label "Configure your server connection below."
        Style := Dim

    Separator

    Group "Server"

        TextBox
            Label       := "Host:"
            Placeholder := "e.g. localhost"
            MaxLength   := 255
            Binding     := serverHost
            OnChange    := HostChanged

        NumberBox
            Label   := "Port:"
            Min     := 1
            Max     := 65535
            Binding := serverPort

        DropDown
            Label   := "Protocol:"
            Options := "HTTP", "HTTPS", "FTP"
            Binding := serverProtocol
            OnChange := ProtocolChanged

    End Group

    Group "Authentication"

        RadioGroup
            Label   := "Auth Type:"
            Options := "None", "Basic", "Token"
            Binding := authType
            OnChange := AuthTypeChanged

        TextBox
            Label   := "Username:"
            Binding := username

        TextBox
            Label   := "Password:"
            Binding := password

        CheckBox
            Label   := "Remember credentials"
            Binding := rememberCredentials
            OnChange := RememberToggled

    End Group

    ProgressBar
        Label   := "Testing connection..."
        Binding := connectionProgress

    Separator

    Row Align := SpaceBetween

        Button "Test Connection"
            OnClick := TestConnection
            Style   := Normal

        Button "Connect"
            OnClick := Connect
            Style   := Primary

        Button "Cancel"
            OnClick := Cancel
            Style   := Danger

    End Row

    StatusBar
        Binding := statusMessage

End Form


' --- Event handlers ---

Function HostChanged(value As String)
    statusMessage = "Host: " & value
End Function

Function ProtocolChanged(value As String)
End Function

Function AuthTypeChanged(value As String)
End Function

Function RememberToggled(value As Bool)
    If value Then
        statusMessage = "Credentials will be saved."
    Else
        statusMessage = "Credentials not saved."
    End If
End Function

Function TestConnection()
    statusMessage = "Testing..."
    connectionProgress = 0.0
End Function

Function Connect()
    statusMessage = "Connecting..."
End Function

Function Cancel()
End Function
