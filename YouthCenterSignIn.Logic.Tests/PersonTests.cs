using System;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn.Logic.Tests
{
    [TestClass]
    public class PersonTests : TestBase
    {
        [TestMethod]
        public void Person_HasValidInfoTest()
        {
            var person = new Person();

            Assert.IsFalse(person.IsValid(out string issues), "The person should not be valid when first created.");
            Assert.AreEqual("Please enter your first name, last name and address.\r\nYou have to be at least five to sign up.",
                issues, "The first name, last name and address are blank and there is an invalid date of birth.");

            person.LastName = "Esh";
            Assert.IsFalse(person.IsValid(out issues), "The person should not be valid when first created.");
            Assert.AreEqual("Please enter your first name and address.\r\nYou have to be at least five to sign up.",
                issues, "The first name and address are blank and there is an invalid date of birth.");

            person.BirthDate = new DateTimeOffset(new DateTime(1999, 2, 23));
            person.Address = new Address("52 Evergreen St", "Gordonville", "PA");
            Assert.IsFalse(person.IsValid(out issues), "The person should not be valid when first created.");
            Assert.AreEqual("Please enter your first name.\r\n", issues, "The first name is blank.");

            person.FirstName = "James";
            Assert.IsTrue(person.IsValid(out issues), "All the issues are resolved.");
            Assert.AreEqual("", issues, "There are no issues.");
        }

        [TestMethod]
        public void Person_HasValidDateTest()
        {
            var person = new Person() { FirstName = "John", LastName = "Doe" };

            person.BirthDate = DateTimeOffset.Now.AddYears(-2);
            Assert.IsFalse(person.IsValid(out string issues), "The person should not be valid when first created.");
            Assert.AreEqual("Please enter your address.\r\nYou have to be at least five to sign up.",
                issues, "The address is blank and there is an invalid date of birth.");

            person.Address = new Address("123 Programmer St", "Geekville", "UT");
            person.BirthDate = DateTimeOffset.Now.AddYears(-5);
            Assert.IsTrue(person.IsValid(out issues), "All the issues are resolved.");
            Assert.AreEqual("", issues, "There are no issues.");

            person.BirthDate = DateTimeOffset.Now.AddYears(-5).AddDays(1);
            Assert.IsFalse(person.IsValid(out issues), "The person should not be valid when first created.");
            Assert.AreEqual("You have to be at least five to sign up.",
                issues, "There is still an invalid date of birth.");

            person.BirthDate = new DateTimeOffset(new DateTime(1999, 2, 23));
            Assert.IsTrue(person.IsValid(out issues), "All the issues are resolved.");
            Assert.AreEqual("", issues, "There are no issues.");
        }

        [TestMethod]
        public void Person_PropertyChangedTest()
        {
            var person = new Person();

            bool changed = false;
            person.PropertyChanged += (_, __) => changed = true;

            person.FirstName = "James";
            Assert.IsTrue(changed, "The property changed event should have fired when a property changed.");
        }

        [TestMethod]
        public void PersonSearch_SaveTest()
        {
            var ogPersonId = "An assigned id";
            var person = new Person()
            {
                Id = ogPersonId,
                FirstName = "John",
                LastName = "Doe",
                Address = new Address("123 Programmer St", "Geekville", "UT"),
                BirthDate = DateTimeOffset.Now.AddYears(-3)
            };

            Assert.ThrowsExceptionAsync<Exception>(async () => await person.Save(),
                "The person isn't valid, so the save should fail.");

            person.BirthDate = DateTimeOffset.Now.AddYears(-10);
            Assert.ThrowsExceptionAsync<Exception>(async () => await person.Save(),
                "There is no guardian, so the save should fail.");

            person.Guardian = new Guardian();
            Assert.ThrowsExceptionAsync<Exception>(async () => await person.Save(),
                "The guardian is invalid, so the save should fail.");

            person.Guardian.Name = "Jane Doe";
            person.Guardian.PhoneNumber = "717-334-3334";
            person.Save().Wait();

            Assert.IsFalse(string.IsNullOrWhiteSpace(person.Id), "The id should have been assigned by the data provider");
            Assert.AreNotEqual(ogPersonId, person.Id, "The id should change to the value assigned by data provider");
        }
    }
}
