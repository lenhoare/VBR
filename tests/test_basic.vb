' Basic types and variable declarations
Dim a As Integer = 5
Dim b As Long = 100
Dim c As Double = 3.14
Dim d As Boolean = True
Dim e As String = "hello"
Dim f As Byte = 255

' Arithmetic operations
Dim sum As Integer = a + b
Dim product As Double = c * 2.5
Dim diff As Integer = a - b

' Control flow
If a > 0 Then
    Dim pos As Boolean = True
End If

If b < 1000 Then
    Dim small As Boolean = True
Else
    Dim large As Boolean = False
End If

' Select Case
Select Case a
    Case 1
        Dim one As Integer = 1
    Case 2
        Dim two As Integer = 2
    Case Else
        Dim other As Integer = 0
End Select

' Loops
For i = 1 To 10
    Dim square As Integer = i * i
Next

For Each item In e
    Dim ch As String = item
Next

While a > 0
    Dim decremented As Integer = a - 1
Wend

Do While a > 0
    Dim doVar As Integer = 0
Loop

Do Until a < 0
    Dim untilVar As Integer = 1
Loop

' Functions
Function Add(x As Integer, y As Integer) As Integer
    Function = x + y
End Function

Function GetName() As String
    GetName = "test"
End Function

Dim result As Integer = Add(3, 4)
Dim name As String = GetName()

' Error handling
On Error Resume Next
Dim risky As Integer = 1 / 0
On Error GoTo 0