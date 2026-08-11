(*************************************************************************

     $RCSfile: NotHere.mod $
  Description: Program to take the place of a program that has moved
               elsewhere.

   Created by: fjc (Frank Copeland)
    $Revision: 1.1 $
      $Author: fjc $
        $Date: 1995/01/26 01:05:15 $

  Copyright © 1995, Frank Copeland.
  This file is part of Oberon-A.
  See Oberon-A.doc for conditions of use and distribution.

  Log entries are at the end of the file.

*************************************************************************)

<* STANDARD- *>

MODULE NotHere;

IMPORT SYS := SYSTEM, Kernel, wb := Workbench, i := Intuition, d := Dos;

CONST
  VersionTag = "$VER: NotHere 1.1 (10.1.95)";

VAR
  progName : ARRAY 32 OF CHAR;
  dirName : ARRAY 256 OF CHAR;
  wbStartup : wb.WBStartupPtr;
  es : i.EasyStruct;
  lock : d.FileLockPtr;
  a1, a2 : SYS.ADDRESS;

BEGIN
  IF Kernel.fromWorkbench THEN
    wbStartup := SYS.VAL (wb.WBStartupPtr, Kernel.WBenchMsg);
    ASSERT
      (d.NameFromLock (wbStartup.argList[0].lock, dirName, LEN (dirName)));
    COPY (wbStartup.argList [0].name^, progName);
  ELSE
    lock := d.GetProgramDir ();
    ASSERT (d.NameFromLock (lock, dirName, LEN (dirName)));
    ASSERT (d.GetProgramName (progName, LEN (progName)));
  END;

  es.structSize := SIZE (i.EasyStruct);
  es.flags := {};
  es.title := SYS.ADR ("Not Here :-(");
  es.gadgetFormat := SYS.ADR ("Oh well...");
  es.textFormat :=
    SYS.ADR ("'%s' doesn't live in directory\n'%s' any more");
  a1 := SYS.ADR (progName); a2 := SYS.ADR (dirName);
  ASSERT (i.EasyRequest ( NIL, SYS.ADR (es), NIL, a1, a2, NIL ) # 0)
END NotHere.

(*************************************************************************

  $Log: NotHere.mod $
# Revision 1.1  1995/01/26  01:05:15  fjc
# - Release 1.5
#
*************************************************************************)
