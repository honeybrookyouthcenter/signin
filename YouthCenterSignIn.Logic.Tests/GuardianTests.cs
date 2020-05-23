using System;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn.Logic.Tests
{
    [TestClass]
    public class GuardianTests
    {
        [TestMethod]
        public void Guardian_IsValidTest()
        {
            var guardian = new Guardian();
            Assert.IsFalse(guardian.IsValid(out string issues), "Everything is blank, so it's not valid");
            Assert.AreEqual(issues, "Please enter your guardian's name and phone number.",
                "There should be a helpful message");

            guardian.Name = "Glenn Esh";
            Assert.IsFalse(guardian.IsValid(out issues), "Everything is blank, so it's not valid");
            Assert.AreEqual(issues, "Please enter your guardian's phone number.",
                "There should be a helpful message");

            guardian.PhoneNumber = "1234567890";
            Assert.IsTrue(guardian.IsValid(out issues), "Everything is filled out, it should be valid.");
        }

        [TestMethod]
        public void Guardian_ToString_SimpleTest()
        {
            var guardian = new Guardian("James Esh", "610 883 2281", "jamesesh@live.com");
            Assert.IsTrue(guardian.ToString().Contains("James Esh"), "The guardian text should contain the name.");
            Assert.IsTrue(guardian.ToString().Contains("610 883 2281"), "The guardian text should contain the phone.");
        }

        [TestMethod]
        public void Guardian_ToString_ComplexTest()
        {
            var guardian = new Guardian("Glenn Esh", "(717) 629-0658", "gesh@eshcom.com");
            Assert.IsTrue(guardian.ToString().Contains("Glenn Esh"), "The guardian text should contain the name.");
            Assert.IsTrue(guardian.ToString().Contains("(717) 629-0658"), "The guardian text should contain the phone.");
            Assert.IsTrue(guardian.ToString().Contains("gesh@eshcom.com"), "The guardian text should contain the email.");
        }

        [TestMethod]
        public void Guardian_FromNotesTest()
        {
            var guardian = new Guardian("James Esh", "6108832281", "jamesesh@live.com");
            AssertNotesAreEqual(guardian, guardian.ToString());
        }

        [TestMethod]
        public void Guardian_FromNotes_ComplexTest()
        {
            var guardian = new Guardian("Glenn :: Esh", "(717) 629-0658", "jamesesh@live.com");
            AssertNotesAreEqual(guardian, guardian.ToString());
        }

        [TestMethod]
        public void Guardian_FromNotes_NoEmailTest()
        {
            var guardian = new Guardian("James Esh", "6108832281");
            AssertNotesAreEqual(guardian, guardian.ToString());
        }

        [TestMethod]
        public void Guardian_FromNotes_EmptyNotesTest()
        {
            var gaurdian = new Guardian("", "");
            AssertNotesAreEqual(gaurdian, "");
        }

        [TestMethod]
        public void Guardian_FromNotes_NullNotesTest()
        {
            var gaurdian = new Guardian("", "");
            AssertNotesAreEqual(gaurdian, null);
        }

        [TestMethod]
        public void Guardian_FromNotes_BadNotesTest()
        {
            var gaurdian = new Guardian("", "");
            AssertNotesAreEqual(gaurdian, $"sdlkfjsl lksdf\r\n{new Guardian().ToString()}sdlfj");
            AssertNotesAreEqual(gaurdian, $"sdlkfjsl lksdf\r\nsdlfj:: =sd+\r\n\tsdfskj");
        }

        void AssertNotesAreEqual(Guardian original, string notes)
        {
            var guardianFromNotes = Guardian.FromNotes(notes);

            void AssertFieldEquals(string expected, string actual) =>
                Assert.AreEqual(expected ?? "", actual, "Saving and reading should not affect it.");

            AssertFieldEquals(original.Name, guardianFromNotes.Name);
            AssertFieldEquals(original.PhoneNumber, guardianFromNotes.PhoneNumber);
            AssertFieldEquals(original.Email, guardianFromNotes.Email);
            AssertFieldEquals(original.LastUpdated.ToString(), guardianFromNotes.LastUpdated.ToString());
        }
    }
}
