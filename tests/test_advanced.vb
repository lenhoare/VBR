' Advanced features
Type Person
    Name As String
    Age As Integer
    IsActive As Boolean
End Type

Dim p As Person
p.Name = "Alice"
p.Age = 30
p.IsActive = True

' Collections
Dim dict As New HashMap<String, Integer>
dict.insert("one", 1)
dict.insert("two", 2)

Dim value As Integer = dict.get("one")
Dim exists As Boolean = dict.contains_key("two")
dict.remove("one")

For Each k, v In dict
    Dim kv As Integer = v
Next

' Constants
Const PI As Double = 3.14159
Const MAX_VALUE As Long = 1000000
Public Const GLOBAL_CONST As Integer = 42

' String operations
Dim s1 As String = "hello"
Dim s2 As String = "world"
Dim combined As String = s1 & " " & s2
Dim upper As String = UCase(s1)
Dim lower As String = LCase(s1)
Dim trimmed As String = Trim("  spaces  ")
Dim found As Boolean = InStr(s1, "e") > 0
Dim replaced As String = Replace(s1, "l", "x")
Dim length As Integer = Len(s1)
Dim leftPart As String = Left(s1, 2)
Dim rightPart As String = Right(s1, 3)
Dim midPart As String = Mid(s1, 2, 3)

' Math functions
Dim sq As Double = Sqr(16.0)
Dim absVal As Integer = Abs(-5)
Dim intVal As Integer = Int(3.7)
Dim roundVal As Integer = Round(3.5)
Dim sinVal As Double = Sin(0.0)
Dim cosVal As Double = Cos(0.0)
Dim logVal As Double = Log(2.71828)
Dim expVal As Double = Exp(1.0)

' Arrays (fixed size)
Dim arr(5) As Integer
Dim matrix(10, 20) As Long

' Result type
Function Divide(a As Double, b As Double) As Result<Double, String>
    If b = 0 Then
        Return Err("Division by zero")
    End If
    Return Ok(a / b)
End Function

Dim divisionResult As Result<Double, String> = Divide(10, 2)

' With statement (will be converted to explicit access)
With p
    Dim pname As String = .Name
    Dim page As Integer = .Age
End With

' Property access
Dim hasName As Boolean = p.Name.Length > 0