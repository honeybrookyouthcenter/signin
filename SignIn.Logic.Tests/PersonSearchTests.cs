using Microsoft.VisualStudio.TestTools.UnitTesting;
using SignIn.Logic.Data;

namespace SignIn.Logic.Tests
{
    [TestClass]
    public class PersonSearchTests : TestBase
    {
        [TestMethod]
        public void PersonSearch_ResultsTest()
        {
            Person person = GetTestPerson();
            Assert.AreEqual("James", person.FirstName,
                "The search text was set to James, so the first (and only) result should have a first name of James.");
        }

        [TestMethod]
        public void PersonSearch_MultipleResultsTest()
        {
            var search = new PersonSearch
            {
                SearchText = "esh"
            };
            search.SearchTask.Wait();
            Assert.AreEqual(2, search.Count, "The search for Esh should have found 2 people.");

            search.SearchText = "James Esh";
            search.SearchTask.Wait();
            Assert.AreEqual(1, search.Count, "The results should be cleared after setting the text to blank.");

            search.SearchText = " ";
            search.SearchTask.Wait();
            Assert.AreEqual(0, search.Count, "The results should be cleared after setting the text to blank.");

            search.SearchText = null;
            search.SearchTask.Wait();
            Assert.AreEqual(0, search.Count, "The results should be cleared after setting the text to null.");
        }

        [TestMethod]
        public void PersonSearch_SortResultsTest()
        {
            var search = new PersonSearch
            {
                SearchText = "peter"
            };
            search.SearchTask.Wait();
            Assert.AreEqual(2, search.Count, "The search for peter should have found 2 people.");

            Assert.AreEqual("Peter", search[0].FirstName,
                "Peter McKinnin should be before Merv Petersheim because the search starts with peter");
        }
    }
}
