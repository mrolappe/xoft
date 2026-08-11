(*************************************************************************

     $RCSfile: HexConvert.mod $
  Description: A little utility to convert unsigned hex integers to their
               signed equivalents.

   Created by: fjc (Frank Copeland)
    $Revision: 1.3 $
      $Author: fjc $
        $Date: 1995/01/26 01:05:15 $

  Log entries are at the end of the file.

*************************************************************************)

<*STANDARD-*>

MODULE HexConvert;

IMPORT IO := StdIO;

CONST
  VersionTag = "$VER: HexConvert 1.1 (24.9.94)\r\n";
  VersionStr = "HexConvert 1.1 (24.9.94)\r\n";
  CopyrightStr = "Written by Frank Copeland\n";

VAR
  i, j, k : INTEGER; temp : LONGINT; ch : CHAR; digit : ARRAY 17 OF CHAR;

BEGIN (* HexConvert *)
  IO.WriteStr (VersionStr);
  IO.WriteStr (CopyrightStr);
  IO.WriteLn ();
  digit := "0123456789ABCDEF";
  LOOP
    i := 0; temp := 0;
    IO.WriteStr ("Enter a hex integer => ");
    LOOP
      IO.Read (ch); IF ch = "\n" THEN EXIT END;
      ch := CAP (ch); INC (i);
      IF (ch >= "0") & (ch <= "9") THEN
        k := ORD (ch) - ORD ("0")
      ELSIF (ch >= "A") & (ch <= "F") THEN
        k := ORD (ch) - (ORD ("A") - 10)
      ELSE
        IO.WriteStr ("\n !! Illegal hex digit\n"); temp := -1; EXIT
      END;
      IF i < 5 THEN temp := temp * 16 + k END
    END;
    IF i = 0 THEN
      EXIT
    ELSIF i > 4 THEN
      IO.WriteStr ("\n !! Number is too big\n"); temp := -1
    END;
    IF temp >= 0 THEN
      IF temp > 32767 THEN DEC (temp, 65536) END;
      IO.WriteStr (" >> Signed equivalent : ");
      IF temp < 0 THEN IO.Write ("-"); temp := ABS (temp) END;
      IO.WriteHex (temp); IO.WriteLn ()
    END;
  END;
  IO.WriteStr ("\nAll finished\n");
END HexConvert.

(*************************************************************************

  $Log: HexConvert.mod $
# Revision 1.3  1995/01/26  01:05:15  fjc
# - Release 1.5
#
# Revision 1.2  1994/09/25  18:25:17  fjc
# - Uses new syntax for external code declarations
#
# Revision 1.1  1994/06/09  14:29:51  fjc
# Initial revision
#
*************************************************************************)


