' Test error cases
' Cannot assign unknown size to fixed variable
' This should cause an error in the parser/type system

' Invalid type
Dim invalid As Currency

' Variant type
Dim variant As Variant

' With statement (not supported)
With p
    .Name = "test"
End With

' Option Base (not supported)
Option Base 1

' ReDim without Preserve (data loss)
Dim arr() As Integer
ReDim arr(10)

' Sub procedures (not supported)
Sub NoReturn()
    ' This should be converted to Function
End Sub

' Pointer types
Dim ptr As LongPtr

' Custom type without initialization
Type MyStruct
    name As String
    value As Integer
End Type

Dim s As MyStruct  ' Should require initialization

' Chained assignments
Dim a As Integer, b As Integer, c As Integer
a = b = c  ' Not supported in Rust

' Call by reference without explicit ByRef
Dim x As Integer
CallFunction x  ' Implicit ByRef