' Vec as first-class citizen in VBR
Dim scores As New Vec<Long>
Dim names As New Vec<String>

' Push elements
scores.push(42)
scores.push(100)
scores.push(7)

' Pop returns Option
Dim last As Long = scores.pop()
Dim maybeEmpty As Long = scores.pop()

' Length
Dim n As Long = scores.len()

' Is empty
If scores.is_empty() Then
    Dim emptyFlag As Boolean = True
End If

' Insert at position
scores.insert(2, 99)

' Remove at position
Dim removed As Long = scores.remove(0)

' Safe access with get
Dim x As Long = scores.get(0)

' Iterating with For Each
For Each score In scores
    Debug.Print score
Next

' Vec literal initialization
Dim values As Vec<Long> = [1, 2, 3, 4, 5]
Dim names2 As Vec<String> = ["Alice", "Bob", "Charlie"]

' Contains
If scores.contains(42) Then
    Dim found As Boolean = True
End If

' Multiple operations
Dim total As Long = 0
For Each val In values
    total = total + val
Next

Function sumVec(v As Vec<Long>) As Long
    Dim sum As Long = 0
    For Each item In v
        sum = sum + item
    Next
    Return sum
End Function

Dim myVec As New Vec<Integer>
myVec.push(1)
myVec.push(2)
Dim item As Integer = myVec.get(0)